from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import lean_suite
import protocol


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


def add_valid_lake(lock: dict[str, object], root: Path) -> None:
    fake_lake = root / "tools" / "fake-lake"
    fake_lake.parent.mkdir(parents=True, exist_ok=True)
    fake_lake.write_text("#!/bin/sh\nprintf 'Lake fake 4.0.0\\n'\n", encoding="utf-8")
    fake_lake.chmod(0o755)
    lock["lake"] = {
        "path": "tools/fake-lake",
        "version": "Lake fake 4.0.0",
        "bytes": fake_lake.stat().st_size,
        "sha256": digest(fake_lake.read_bytes()),
    }


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


class LeanSuiteValidationTests(unittest.TestCase):
    def test_runtime_lock_rejects_unsupported_schema_and_nested_unknowns(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            lock = valid_lock(root)
            lock["schema_version"] = "crouzeix-lean-runtime-lock/v2"
            with self.assertRaisesRegex(protocol.ValidationError, "schema_version"):
                lean_suite.validate_runtime_lock(lock, root)

            nested = valid_lock(root)
            nested["lean"] = dict(nested["lean"])
            nested["lean"]["extra"] = True
            with self.assertRaisesRegex(protocol.ValidationError, "lean fields"):
                lean_suite.validate_runtime_lock(nested, root)

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

    def test_runtime_lock_rejects_bad_environment_and_profiles(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            lock = valid_lock(root)
            lock["allowed_env"] = ["PATH"]
            with self.assertRaisesRegex(protocol.ValidationError, "allowed_env"):
                lean_suite.validate_runtime_lock(lock, root)

            bad_profile = valid_lock(root)
            bad_profile["command_profiles"] = [
                {
                    "profile_id": "lean-check",
                    "argv": ["{lean}", "{module_path}"],
                    "timeout_seconds": 10,
                    "env": {"PATH": "/bin"},
                }
            ]
            with self.assertRaisesRegex(protocol.ValidationError, "env"):
                lean_suite.validate_runtime_lock(bad_profile, root)

            missing_token = valid_lock(root)
            missing_token["command_profiles"] = [
                {
                    "profile_id": "lean-check",
                    "argv": ["lean", "{module_path}"],
                    "timeout_seconds": 10,
                    "env": {},
                }
            ]
            with self.assertRaisesRegex(protocol.ValidationError, "argv"):
                lean_suite.validate_runtime_lock(missing_token, root)

    def test_runtime_lock_rejects_command_profile_with_locked_tool_after_ambient_wrapper(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            bad_profile = valid_lock(root)
            bad_profile["command_profiles"] = [
                {
                    "profile_id": "lean-check",
                    "argv": ["python3", "{lean}", "{module_path}"],
                    "timeout_seconds": 10,
                    "env": {},
                }
            ]
            with self.assertRaisesRegex(
                protocol.ValidationError, "argv must start from a locked tool token"
            ):
                lean_suite.validate_runtime_lock(bad_profile, root)

    def test_runtime_lock_rejects_loader_and_python_runtime_environment(self) -> None:
        blocked = [
            "DYLD_INSERT_LIBRARIES",
            "DYLD_LIBRARY_PATH",
            "LD_PRELOAD",
            "LD_LIBRARY_PATH",
            "PYTHONPATH",
            "PYTHONHOME",
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            for name in blocked:
                with self.subTest(name=name):
                    lock = valid_lock(root)
                    lock["allowed_env"] = [name]
                    with self.assertRaisesRegex(protocol.ValidationError, "allowed_env"):
                        lean_suite.validate_runtime_lock(lock, root)

    def test_runtime_lock_allows_explicit_harmless_environment(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            lock = valid_lock(root)
            lock["allowed_env"] = ["LEAN_SUITE_CACHE"]
            lock["command_profiles"] = [
                {
                    "profile_id": "lean-check",
                    "argv": ["{lean}", "{module_path}"],
                    "timeout_seconds": 10,
                    "env": {"LEAN_SUITE_CACHE": "tmp/lean-suite-cache"},
                }
            ]

            runtime = lean_suite.validate_runtime_lock(lock, root)

            self.assertEqual(
                {"LEAN_SUITE_CACHE": "tmp/lean-suite-cache"},
                runtime.command_profiles["lean-check"].env,
            )

    def test_runtime_lock_rejects_lake_profile_without_locked_lake_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            lock = valid_lock(root)
            lock["command_profiles"] = [
                {
                    "profile_id": "lake-build",
                    "argv": ["{lake}", "build", "{module_name}"],
                    "timeout_seconds": 10,
                    "env": {},
                }
            ]
            with self.assertRaisesRegex(protocol.ValidationError, "lake"):
                lean_suite.validate_runtime_lock(lock, root)

    def test_runtime_lock_allows_lake_profile_with_locked_lake_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            lock = valid_lock(root)
            add_valid_lake(lock, root)
            lock["command_profiles"] = [
                {
                    "profile_id": "lake-build",
                    "argv": ["{lake}", "build", "{module_name}"],
                    "timeout_seconds": 10,
                    "env": {},
                }
            ]

            runtime = lean_suite.validate_runtime_lock(lock, root)

            self.assertIsNotNone(runtime.lake)
            self.assertEqual(
                ("{lake}", "build", "{module_name}"),
                runtime.command_profiles["lake-build"].argv,
            )


class LeanSuiteManifestTests(unittest.TestCase):
    def test_manifest_rejects_unsupported_schema_and_nested_unknowns(self) -> None:
        manifest = valid_manifest()
        manifest["schema_version"] = "crouzeix-lean-suite-manifest/v2"
        with self.assertRaisesRegex(protocol.ValidationError, "schema_version"):
            lean_suite.validate_suite_manifest(manifest, {"lean-check"})

        nested = valid_manifest()
        nested["modules"] = [dict(nested["modules"][0])]
        nested["modules"][0]["extra"] = True
        with self.assertRaisesRegex(protocol.ValidationError, "module fields"):
            lean_suite.validate_suite_manifest(nested, {"lean-check"})

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

    def test_manifest_rejects_duplicate_paths_and_malformed_module_names(self) -> None:
        duplicate_path = valid_manifest()
        second = dict(duplicate_path["modules"][0])
        second["module_id"] = "tiny-smoke-two"
        duplicate_path["modules"] = [duplicate_path["modules"][0], second]
        with self.assertRaisesRegex(protocol.ValidationError, "duplicate module path"):
            lean_suite.validate_suite_manifest(duplicate_path, {"lean-check"})

        malformed = valid_manifest()
        malformed["modules"] = [dict(malformed["modules"][0])]
        malformed["modules"][0]["module_name"] = "tinySmoke"
        with self.assertRaisesRegex(protocol.ValidationError, "module_name"):
            lean_suite.validate_suite_manifest(malformed, {"lean-check"})

        bad_digest = valid_manifest()
        bad_digest["modules"] = [dict(bad_digest["modules"][0])]
        bad_digest["modules"][0]["source_sha256"] = "A" * 64
        with self.assertRaisesRegex(protocol.ValidationError, "source_sha256"):
            lean_suite.validate_suite_manifest(bad_digest, {"lean-check"})

    def test_canonical_digest_is_deterministic(self) -> None:
        left = {"b": [2, 1], "a": {"x": "y"}}
        right = json.loads(json.dumps(left, sort_keys=True))

        self.assertEqual(lean_suite.canonical_sha256(left), lean_suite.canonical_sha256(right))
        self.assertEqual(64, len(lean_suite.canonical_sha256(left)))


if __name__ == "__main__":
    unittest.main()
