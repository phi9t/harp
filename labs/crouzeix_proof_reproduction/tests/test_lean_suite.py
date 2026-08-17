from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
import stat


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))
FIXTURES = LAB / "lean_suite_fixtures"

import lean_suite
import protocol


def digest(data: bytes) -> str:
    return protocol.sha256_bytes(data)


def valid_lock(root: Path) -> dict[str, object]:
    fake_lean = root / "tools" / "fake-lean"
    fake_lean.parent.mkdir(parents=True, exist_ok=True)
    fake_lean.write_text(
        f"#!{sys.executable}\n"
        "from pathlib import Path\n"
        "import sys\n"
        "if '--version' in sys.argv:\n"
        "    print('Lean fake 4.0.0')\n"
        "    raise SystemExit(0)\n"
        "text = Path(sys.argv[1]).read_text(encoding='utf-8')\n"
        "if 'LEAN_SUITE_FAIL' in text:\n"
        "    print('fake lean: type mismatch', file=sys.stderr)\n"
        "    raise SystemExit(1)\n"
        "print(f'fake lean checked {Path(sys.argv[1]).name}')\n",
        encoding="utf-8",
    )
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
    fake_lake.write_text(
        f"#!{sys.executable}\n"
        "import sys\n"
        "if '--version' in sys.argv:\n"
        "    print('Lake fake 4.0.0')\n"
        "    raise SystemExit(0)\n"
        "print('Lake fake 4.0.0')\n",
        encoding="utf-8",
    )
    fake_lake.chmod(0o755)
    lock["lake"] = {
        "path": "tools/fake-lake",
        "version": "Lake fake 4.0.0",
        "bytes": fake_lake.stat().st_size,
        "sha256": digest(fake_lake.read_bytes()),
    }


def replace_fake_lean(lock: dict[str, object], root: Path, body: str) -> None:
    fake_lean = root / "tools" / "fake-lean"
    fake_lean.write_text(f"#!{sys.executable}\n{body}", encoding="utf-8")
    fake_lean.chmod(0o755)
    lock["lean"] = dict(lock["lean"])
    lock["lean"]["bytes"] = fake_lean.stat().st_size
    lock["lean"]["sha256"] = digest(fake_lean.read_bytes())


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


def declared_source(path: Path, relative_path: str, role: str) -> dict[str, object]:
    data = path.read_bytes()
    return {
        "path": relative_path,
        "role": role,
        "sha256": digest(data),
        "bytes": len(data),
    }


def fixture_manifest() -> dict[str, object]:
    smoke = FIXTURES / "smoke" / "TinySmoke.lean"
    lake_module = FIXTURES / "lake" / "LeanSuiteLake.lean"
    manifest = valid_manifest()
    manifest["suite_id"] = "local-fixture-suite"
    manifest["modules"] = [
        {
            "module_id": "tiny-smoke",
            "tier": "tiny_smoke",
            "path": "smoke/TinySmoke.lean",
            "module_name": "TinySmoke",
            "command_profile": "lean-check",
            "expected_declarations": ["tiny_smoke"],
            "allowed_axioms": [],
            "source_sha256": digest(smoke.read_bytes()),
            "source_bytes": smoke.stat().st_size,
        },
        {
            "module_id": "lean-suite-lake",
            "tier": "local_lake",
            "path": "lake/LeanSuiteLake.lean",
            "module_name": "LeanSuiteLake",
            "command_profile": "lake-build",
            "expected_declarations": ["lean_suite_lake_smoke"],
            "allowed_axioms": [],
            "source_sha256": digest(lake_module.read_bytes()),
            "source_bytes": lake_module.stat().st_size,
        },
    ]
    manifest["source_files"] = [
        declared_source(
            FIXTURES / "lake" / "lakefile.toml",
            "lake/lakefile.toml",
            "lake_config",
        ),
        declared_source(
            FIXTURES / "lake" / "lean-toolchain",
            "lake/lean-toolchain",
            "toolchain_lock",
        ),
        declared_source(
            FIXTURES / "tools" / "fake-lean.py",
            "tools/fake-lean.py",
            "adapter",
        ),
        declared_source(
            FIXTURES / "tools" / "fake-lake.py",
            "tools/fake-lake.py",
            "adapter",
        ),
    ]
    return manifest


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

    def test_runtime_lock_rejects_noncanonical_tool_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            for path in ("./tools/fake-lean", "tools//fake-lean"):
                with self.subTest(path=path):
                    lock = valid_lock(root)
                    lock["lean"] = dict(lock["lean"])
                    lock["lean"]["path"] = path
                    with self.assertRaisesRegex(protocol.ValidationError, "lean.path"):
                        lean_suite.validate_runtime_lock(lock, root)

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

    def test_manifest_rejects_noncanonical_module_paths(self) -> None:
        for path in ("./TinySmoke.lean", "Proofs//Main.lean"):
            with self.subTest(path=path):
                manifest = valid_manifest()
                manifest["modules"] = [dict(manifest["modules"][0])]
                manifest["modules"][0]["path"] = path
                with self.assertRaisesRegex(protocol.ValidationError, "module path"):
                    lean_suite.validate_suite_manifest(manifest, {"lean-check"})

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

    def test_manifest_rejects_canonical_equivalent_duplicate_module_paths(self) -> None:
        duplicate_path = valid_manifest()
        second = dict(duplicate_path["modules"][0])
        second["module_id"] = "tiny-smoke-two"
        second["path"] = "./TinySmoke.lean"
        duplicate_path["modules"] = [duplicate_path["modules"][0], second]

        with self.assertRaisesRegex(protocol.ValidationError, "module path"):
            lean_suite.validate_suite_manifest(duplicate_path, {"lean-check"})

    def test_canonical_digest_is_deterministic(self) -> None:
        left = {"b": [2, 1], "a": {"x": "y"}}
        right = json.loads(json.dumps(left, sort_keys=True))

        self.assertEqual(lean_suite.canonical_sha256(left), lean_suite.canonical_sha256(right))
        self.assertEqual(64, len(lean_suite.canonical_sha256(left)))


class LeanSuiteInventoryTests(unittest.TestCase):
    def test_source_inventory_accepts_declared_fixture_sources_deterministically(self) -> None:
        manifest = fixture_manifest()
        parsed = lean_suite.validate_suite_manifest(manifest, {"lean-check", "lake-build"})

        inventory = lean_suite.compute_source_inventory(FIXTURES, parsed)
        repeated = lean_suite.compute_source_inventory(FIXTURES, parsed)

        self.assertEqual(inventory, repeated)
        self.assertEqual("crouzeix-lean-source-inventory/v1", inventory["schema_version"])
        self.assertEqual(6, inventory["file_count"])
        self.assertEqual(
            [
                "lake/LeanSuiteLake.lean",
                "lake/lakefile.toml",
                "lake/lean-toolchain",
                "smoke/TinySmoke.lean",
                "tools/fake-lake.py",
                "tools/fake-lean.py",
            ],
            [file["path"] for file in inventory["files"]],
        )
        self.assertEqual(64, len(lean_suite.canonical_sha256(inventory)))

    def test_source_inventory_rejects_missing_extra_digest_and_byte_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "TinySmoke.lean"
            original = "theorem tiny_smoke : True := by\n  trivial\n"
            source.write_text(original, encoding="utf-8")
            manifest = valid_manifest()
            manifest["modules"] = [dict(manifest["modules"][0])]
            manifest["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest["modules"][0]["source_bytes"] = source.stat().st_size
            parsed = lean_suite.validate_suite_manifest(manifest, {"lean-check"})

            inventory = lean_suite.compute_source_inventory(suite, parsed)
            self.assertEqual(1, inventory["file_count"])

            source.write_text(
                "theorem tiny_smoke : True := by\n  exact True.intro\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(protocol.ValidationError, "source_bytes"):
                lean_suite.compute_source_inventory(suite, parsed)

            source.write_text("axiom tiny_smoke : True\n                 \n", encoding="utf-8")
            self.assertEqual(source.stat().st_size, len(original.encode("utf-8")))
            with self.assertRaisesRegex(protocol.ValidationError, "source_sha256"):
                lean_suite.compute_source_inventory(suite, parsed)

            source.write_text(original, encoding="utf-8")
            (suite / "Extra.lean").write_text(
                "theorem extra : True := by\n  trivial\n", encoding="utf-8"
            )
            with self.assertRaisesRegex(protocol.ValidationError, "unknown source"):
                lean_suite.compute_source_inventory(suite, parsed)

            (suite / "Extra.lean").unlink()
            source.unlink()
            with self.assertRaisesRegex(protocol.ValidationError, "missing source"):
                lean_suite.compute_source_inventory(suite, parsed)

    def test_source_inventory_rejects_symlinked_source_and_suite_root(self) -> None:
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

            real_suite = root / "real-suite"
            real_suite.mkdir()
            (real_suite / "TinySmoke.lean").write_text(
                target.read_text(encoding="utf-8"),
                encoding="utf-8",
            )
            symlinked_suite = root / "symlinked-suite"
            symlinked_suite.symlink_to(real_suite, target_is_directory=True)
            with self.assertRaisesRegex(protocol.ValidationError, "suite root"):
                lean_suite.compute_source_inventory(symlinked_suite, parsed)

    def test_source_inventory_rejects_symlinked_suite_ancestor(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            real_parent = root / "real-parent"
            real_suite = real_parent / "suite"
            real_suite.mkdir(parents=True)
            source = real_suite / "TinySmoke.lean"
            source.write_text("theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8")
            manifest = valid_manifest()
            manifest["modules"] = [dict(manifest["modules"][0])]
            manifest["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest["modules"][0]["source_bytes"] = source.stat().st_size
            parsed = lean_suite.validate_suite_manifest(manifest, {"lean-check"})
            symlink_parent = root / "symlink-parent"
            symlink_parent.symlink_to(real_parent, target_is_directory=True)

            with self.assertRaisesRegex(protocol.ValidationError, "suite root"):
                lean_suite.compute_source_inventory(symlink_parent / "suite", parsed)

    def test_fixture_fake_tools_are_regular_executable_provider_free_files(self) -> None:
        for relative in ("tools/fake-lean.py", "tools/fake-lake.py"):
            with self.subTest(relative=relative):
                path = FIXTURES / relative
                mode = path.stat().st_mode
                text = path.read_text(encoding="utf-8")
                self.assertTrue(stat.S_ISREG(mode))
                self.assertFalse(path.is_symlink())
                self.assertTrue(mode & stat.S_IXUSR)
                self.assertNotIn("subprocess", text)
                self.assertNotIn("os.system", text)
                self.assertNotIn("Popen", text)
                self.assertNotIn("shutil.which", text)


class LeanSuiteDigestBindingTests(unittest.TestCase):
    def test_runtime_lock_digest_binds_allowed_env_and_created_at(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            baseline = lean_suite.validate_runtime_lock(valid_lock(root), root)

            env_changed_value = valid_lock(root)
            env_changed_value["allowed_env"] = ["LEAN_SUITE_CACHE"]
            env_changed = lean_suite.validate_runtime_lock(env_changed_value, root)

            timestamp_changed_value = valid_lock(root)
            timestamp_changed_value["created_at_utc"] = "2026-08-16T12:00:01Z"
            timestamp_changed = lean_suite.validate_runtime_lock(
                timestamp_changed_value, root
            )

            baseline_digest = lean_suite.canonical_sha256(
                lean_suite._runtime_lock_wire(baseline)
            )
            self.assertNotEqual(
                baseline_digest,
                lean_suite.canonical_sha256(
                    lean_suite._runtime_lock_wire(env_changed)
                ),
            )
            self.assertNotEqual(
                baseline_digest,
                lean_suite.canonical_sha256(
                    lean_suite._runtime_lock_wire(timestamp_changed)
                ),
            )

    def test_suite_manifest_digest_binds_created_at_and_source_files(self) -> None:
        baseline = lean_suite.validate_suite_manifest(valid_manifest(), {"lean-check"})

        timestamp_changed_value = valid_manifest()
        timestamp_changed_value["created_at_utc"] = "2026-08-16T12:00:01Z"
        timestamp_changed = lean_suite.validate_suite_manifest(
            timestamp_changed_value, {"lean-check"}
        )

        source_files_changed_value = valid_manifest()
        source_files_changed_value["source_files"] = [
            {
                "path": "lakefile.toml",
                "role": "lake_config",
                "sha256": "b" * 64,
                "bytes": 0,
            }
        ]
        source_files_changed = lean_suite.validate_suite_manifest(
            source_files_changed_value, {"lean-check"}
        )

        baseline_digest = lean_suite.canonical_sha256(
            lean_suite._suite_manifest_wire(baseline)
        )
        self.assertNotEqual(
            baseline_digest,
            lean_suite.canonical_sha256(
                lean_suite._suite_manifest_wire(timestamp_changed)
            ),
        )
        self.assertNotEqual(
            baseline_digest,
            lean_suite.canonical_sha256(
                lean_suite._suite_manifest_wire(source_files_changed)
            ),
        )


class LeanSuiteRunnerTests(unittest.TestCase):
    def test_runner_records_passed_failed_and_blocked_outcomes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "TinySmoke.lean"
            source.write_text(
                "theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8"
            )
            lock_value = valid_lock(root)
            manifest_value = valid_manifest()
            manifest_value["modules"] = [dict(manifest_value["modules"][0])]
            manifest_value["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest_value["modules"][0]["source_bytes"] = source.stat().st_size
            lock = lean_suite.validate_runtime_lock(lock_value, root)
            manifest = lean_suite.validate_suite_manifest(
                manifest_value, set(lock.command_profiles)
            )

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
            self.assertEqual({}, receipt["command_receipts"][0]["env"])
            self.assertNotIn("PATH", receipt["command_receipts"][0]["env"])
            self.assertNotEqual(
                suite.as_posix(), receipt["command_receipts"][0]["cwd"]
            )
            self.assertNotEqual(
                (suite / "TinySmoke.lean").as_posix(),
                receipt["command_receipts"][0]["argv"][1],
            )
            self.assertTrue(
                receipt["command_receipts"][0]["argv"][1].startswith(
                    receipt["command_receipts"][0]["cwd"] + "/"
                )
            )
            for field in (
                "runtime_inventory_sha256",
                "artifact_inventory_sha256",
                "execution_root_sha256",
            ):
                self.assertIn(field, receipt)
                self.assertEqual(64, len(receipt[field]))

            source.write_text("LEAN_SUITE_FAIL\n", encoding="utf-8")
            manifest_value["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest_value["modules"][0]["source_bytes"] = source.stat().st_size
            manifest = lean_suite.validate_suite_manifest(
                manifest_value, set(lock.command_profiles)
            )
            failed_path = lean_suite.run_suite(
                runtime_lock=lock,
                manifest=manifest,
                suite_root=suite,
                receipt_root=root / "receipts",
                run_id="lean-suite-fail",
            )
            failed = lean_suite.validate_receipt_json(failed_path)
            self.assertEqual(failed["outcome"], "failed")
            self.assertEqual("failed", failed["module_outcomes"][0]["outcome"])
            self.assertNotEqual(0, failed["command_receipts"][0]["exit_code"])

            Path(lock.lean.absolute_path).unlink()
            blocked_path = lean_suite.run_suite(
                runtime_lock=lock,
                manifest=manifest,
                suite_root=suite,
                receipt_root=root / "receipts",
                run_id="lean-suite-blocked",
            )
            blocked = lean_suite.validate_receipt_json(blocked_path)
            self.assertEqual(blocked["outcome"], "blocked")
            self.assertEqual("blocked", blocked["module_outcomes"][0]["outcome"])
            self.assertIsNone(blocked["command_receipts"][0]["exit_code"])

    def test_runner_blocks_drifted_locked_tool_before_invocation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "TinySmoke.lean"
            source.write_text(
                "theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8"
            )
            sentinel = root / "would-have-run.txt"
            lock_value = valid_lock(root)
            manifest_value = valid_manifest()
            manifest_value["modules"] = [dict(manifest_value["modules"][0])]
            manifest_value["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest_value["modules"][0]["source_bytes"] = source.stat().st_size
            lock = lean_suite.validate_runtime_lock(lock_value, root)
            manifest = lean_suite.validate_suite_manifest(
                manifest_value, set(lock.command_profiles)
            )
            drifted_tool = Path(lock.lean.absolute_path)
            drifted_tool.write_text(
                f"#!{sys.executable}\n"
                "from pathlib import Path\n"
                f"Path({str(sentinel)!r}).write_text('invoked\\n', encoding='utf-8')\n"
                "print('drifted tool invoked')\n",
                encoding="utf-8",
            )
            drifted_tool.chmod(0o755)

            receipt_path = lean_suite.run_suite(
                runtime_lock=lock,
                manifest=manifest,
                suite_root=suite,
                receipt_root=root / "receipts",
                run_id="lean-suite-drifted-tool",
            )
            receipt = lean_suite.validate_receipt_json(receipt_path)

            self.assertFalse(sentinel.exists())
            self.assertEqual("blocked", receipt["outcome"])
            self.assertEqual("blocked", receipt["module_outcomes"][0]["outcome"])
            self.assertIsNone(receipt["command_receipts"][0]["exit_code"])
            self.assertIn("runtime inventory", receipt["reason"])

    def test_runner_blocks_profile_absolute_write_env_before_invocation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "TinySmoke.lean"
            source.write_text(
                "theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8"
            )
            outside = root / "outside-write.txt"
            lock_value = valid_lock(root)
            lock_value["allowed_env"] = ["LEAN_SUITE_CACHE"]
            lock_value["command_profiles"] = [
                {
                    "profile_id": "lean-check",
                    "argv": ["{lean}", "{module_path}"],
                    "timeout_seconds": 10,
                    "env": {"LEAN_SUITE_CACHE": outside.as_posix()},
                }
            ]
            replace_fake_lean(
                lock_value,
                root,
                "import os\n"
                "from pathlib import Path\n"
                "target = os.environ.get('LEAN_SUITE_CACHE')\n"
                "if target:\n"
                "    Path(target).write_text('outside write\\n', encoding='utf-8')\n"
                "print('outside write attempted')\n",
            )
            manifest_value = valid_manifest()
            manifest_value["modules"] = [dict(manifest_value["modules"][0])]
            manifest_value["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest_value["modules"][0]["source_bytes"] = source.stat().st_size
            lock = lean_suite.validate_runtime_lock(lock_value, root)
            manifest = lean_suite.validate_suite_manifest(
                manifest_value, set(lock.command_profiles)
            )

            receipt_path = lean_suite.run_suite(
                runtime_lock=lock,
                manifest=manifest,
                suite_root=suite,
                receipt_root=root / "receipts",
                run_id="lean-suite-outside-write-env",
            )
            receipt = lean_suite.validate_receipt_json(receipt_path)

            self.assertFalse(outside.exists())
            self.assertEqual("blocked", receipt["outcome"])
            self.assertEqual("blocked", receipt["module_outcomes"][0]["outcome"])
            self.assertIsNone(receipt["command_receipts"][0]["exit_code"])
            self.assertIn("write policy", receipt["reason"])
            self.assertIn("local_process_no_os_sandbox", receipt["reason"])

    def test_runner_detects_materialized_source_mutation_without_touching_original(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "TinySmoke.lean"
            original = "theorem tiny_smoke : True := by\n  trivial\n"
            source.write_text(original, encoding="utf-8")
            lock_value = valid_lock(root)
            replace_fake_lean(
                lock_value,
                root,
                "from pathlib import Path\n"
                "import sys\n"
                "Path(sys.argv[1]).write_text('LEAN_SUITE_MUTATED\\n', encoding='utf-8')\n"
                "print('mutated materialized source')\n",
            )
            manifest_value = valid_manifest()
            manifest_value["modules"] = [dict(manifest_value["modules"][0])]
            manifest_value["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest_value["modules"][0]["source_bytes"] = source.stat().st_size
            lock = lean_suite.validate_runtime_lock(lock_value, root)
            manifest = lean_suite.validate_suite_manifest(
                manifest_value, set(lock.command_profiles)
            )

            receipt_path = lean_suite.run_suite(
                runtime_lock=lock,
                manifest=manifest,
                suite_root=suite,
                receipt_root=root / "receipts",
                run_id="lean-suite-mutates-source",
            )
            receipt = lean_suite.validate_receipt_json(receipt_path)

            self.assertEqual(original, source.read_text(encoding="utf-8"))
            self.assertEqual("failed", receipt["outcome"])
            self.assertEqual("failed", receipt["module_outcomes"][0]["outcome"])
            self.assertIn("execution root inventory", receipt["reason"])

    def test_runner_rejects_undeclared_execution_root_file_after_command(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "TinySmoke.lean"
            source.write_text(
                "theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8"
            )
            lock_value = valid_lock(root)
            replace_fake_lean(
                lock_value,
                root,
                "from pathlib import Path\n"
                "Path.cwd().joinpath('Extra.lean').write_text("
                "'theorem extra : True := by\\n  trivial\\n', encoding='utf-8')\n"
                "print('wrote undeclared file')\n",
            )
            manifest_value = valid_manifest()
            manifest_value["modules"] = [dict(manifest_value["modules"][0])]
            manifest_value["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest_value["modules"][0]["source_bytes"] = source.stat().st_size
            lock = lean_suite.validate_runtime_lock(lock_value, root)
            manifest = lean_suite.validate_suite_manifest(
                manifest_value, set(lock.command_profiles)
            )

            receipt_path = lean_suite.run_suite(
                runtime_lock=lock,
                manifest=manifest,
                suite_root=suite,
                receipt_root=root / "receipts",
                run_id="lean-suite-extra-file",
            )
            receipt = lean_suite.validate_receipt_json(receipt_path)

            self.assertFalse((suite / "Extra.lean").exists())
            self.assertEqual("failed", receipt["outcome"])
            self.assertEqual("failed", receipt["module_outcomes"][0]["outcome"])
            self.assertIn("execution root inventory", receipt["reason"])

    def test_runner_rejects_existing_receipt_output_escape_and_receipt_tampering(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "TinySmoke.lean"
            source.write_text(
                "theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8"
            )
            lock = lean_suite.validate_runtime_lock(valid_lock(root), root)
            manifest_value = valid_manifest()
            manifest_value["modules"] = [dict(manifest_value["modules"][0])]
            manifest_value["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest_value["modules"][0]["source_bytes"] = source.stat().st_size
            manifest = lean_suite.validate_suite_manifest(
                manifest_value, set(lock.command_profiles)
            )
            receipt_root = root / "receipts"
            lean_suite.run_suite(lock, manifest, suite, receipt_root, "lean-suite-pass")
            with self.assertRaisesRegex(protocol.ValidationError, "already exists"):
                lean_suite.run_suite(
                    lock, manifest, suite, receipt_root, "lean-suite-pass"
                )

            with self.assertRaisesRegex(protocol.ValidationError, "receipt root"):
                lean_suite.run_suite(
                    lock,
                    manifest,
                    suite,
                    root / "receipts" / ".." / "escaped",
                    "lean-suite-escape",
                )

            receipt_path = receipt_root / "lean-suite-pass" / "receipt.json"
            receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
            for field in (
                "runtime_inventory_sha256",
                "artifact_inventory_sha256",
                "execution_root_sha256",
            ):
                missing = dict(receipt)
                missing.pop(field, None)
                missing["lean_suite_receipt_sha256"] = (
                    lean_suite.canonical_sha256_without_receipt_self(missing)
                )
                receipt_path.write_text(
                    json.dumps(missing, indent=2, sort_keys=True) + "\n",
                    encoding="utf-8",
                )
                with self.subTest(missing=field):
                    with self.assertRaisesRegex(protocol.ValidationError, "fields"):
                        lean_suite.validate_receipt_json(receipt_path)

                tampered = dict(receipt)
                tampered[field] = "0" * 64
                receipt_path.write_text(
                    json.dumps(tampered, indent=2, sort_keys=True) + "\n",
                    encoding="utf-8",
                )
                with self.subTest(tampered=field):
                    with self.assertRaises(protocol.ValidationError):
                        lean_suite.validate_receipt_json(receipt_path)

            receipt["outcome"] = "blocked"
            receipt_path.write_text(
                json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
            )
            with self.assertRaisesRegex(
                protocol.ValidationError, "lean_suite_receipt_sha256"
            ):
                lean_suite.validate_receipt_json(receipt_path)

            receipt["lean_suite_receipt_sha256"] = (
                lean_suite.canonical_sha256_without_receipt_self(receipt)
            )
            receipt_path.write_text(
                json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
            )
            with self.assertRaisesRegex(protocol.ValidationError, "outcome"):
                lean_suite.validate_receipt_json(receipt_path)

            receipt["unknown"] = True
            receipt["lean_suite_receipt_sha256"] = (
                lean_suite.canonical_sha256_without_receipt_self(receipt)
            )
            receipt_path.write_text(
                json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
            )
            with self.assertRaisesRegex(protocol.ValidationError, "fields"):
                lean_suite.validate_receipt_json(receipt_path)


if __name__ == "__main__":
    unittest.main()
