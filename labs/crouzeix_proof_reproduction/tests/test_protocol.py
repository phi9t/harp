from __future__ import annotations

import hashlib
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import protocol
import prepare_run


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_spec() -> dict[str, object]:
    prompt = b"historical prompt fixture"
    return {
        "schema_version": "crouzeix-run-spec/v1",
        "run_id": "historical-001",
        "arm": "historical",
        "leakage": "L1",
        "model": "gpt-5.6-sol",
        "cli": {
            "path": "/opt/traecli",
            "version": "traecli 0.200.19(internal edition)",
            "sha256": "a" * 64,
        },
        "historical_prompt": {
            "source_bytes": len(prompt),
            "source_sha256": digest(prompt),
            "execution_bytes": len(prompt),
            "execution_sha256": digest(prompt),
            "normalization": "test fixture",
        },
        "sandbox": "workspace-write",
        "approval_policy": "never",
        "allowed_tools": ["Read", "Write"],
        "network_access": False,
        "timeout_seconds": 3600,
        "max_calls": 1,
        "token_accounting": {
            "boundary": "all provider calls launched by this run",
        },
        "generation_visible_files": ["inputs/execution_prompt.txt"],
        "generation_excluded_classes": [
            "public proof manuscripts",
            "Harp Crouzeix packet",
        ],
        "created_at_utc": "2026-08-15T04:00:00Z",
    }


def route_event(
    sequence: int,
    route_id: str,
    from_state: str | None,
    to_state: str,
    *,
    mechanism: str = "power-family recurrence",
    concrete_artifacts: list[str] | None = None,
    candidate_sha256: str | None = None,
    unproved_obligations: list[dict[str, str]] | None = None,
    unresolved_critical_findings: int = 0,
) -> dict[str, object]:
    return {
        "schema_version": "crouzeix-route-event/v1",
        "sequence": sequence,
        "run_id": "orchestrated-001",
        "route_id": route_id,
        "family": "operator-theory",
        "from_state": from_state,
        "to_state": to_state,
        "parent_route_ids": [],
        "prompt_sha256": "b" * 64,
        "output_sha256": "c" * 64,
        "candidate_sha256": candidate_sha256,
        "reason": "test transition",
        "mechanism": mechanism,
        "concrete_artifacts": concrete_artifacts or [],
        "unproved_obligations": unproved_obligations or [],
        "unresolved_critical_findings": unresolved_critical_findings,
        "occurred_at_utc": "2026-08-15T04:00:00Z",
    }


class HistoricalPromptTests(unittest.TestCase):
    def test_production_prompt_identity_is_pinned(self) -> None:
        self.assertEqual(protocol.HISTORICAL_PROMPT_BYTES, 4106)
        self.assertEqual(
            protocol.HISTORICAL_PROMPT_SHA256,
            "0a0c3000b81efc4d9edc65ec3cd1d53df0d4e69b24bfee9fe0860301d853d6fc",
        )

    def test_verifier_accepts_only_the_declared_size_and_digest(self) -> None:
        fixture = b"proof prompt"

        observed = protocol.verify_historical_prompt(
            fixture,
            expected_bytes=len(fixture),
            expected_sha256=digest(fixture),
        )

        self.assertEqual(
            observed,
            {"bytes": len(fixture), "sha256": digest(fixture)},
        )
        with self.assertRaisesRegex(protocol.ValidationError, "byte count"):
            protocol.verify_historical_prompt(
                fixture + b"x",
                expected_bytes=len(fixture),
                expected_sha256=digest(fixture),
            )
        with self.assertRaisesRegex(protocol.ValidationError, "SHA-256"):
            protocol.verify_historical_prompt(
                fixture,
                expected_bytes=len(fixture),
                expected_sha256="0" * 64,
            )

    def test_normalization_replaces_only_the_nonportable_output_path(self) -> None:
        before = (
            b"keep before "
            + protocol.HISTORICAL_OUTPUT_PATH.encode()
            + b" keep after"
        )

        after, receipt = protocol.normalize_historical_prompt(before)

        self.assertEqual(after, b"keep before candidate.tex keep after")
        self.assertEqual(receipt["replacement_count"], 1)
        self.assertEqual(receipt["source_sha256"], digest(before))
        self.assertEqual(receipt["execution_sha256"], digest(after))
        self.assertNotEqual(receipt["source_sha256"], receipt["execution_sha256"])
        with self.assertRaisesRegex(protocol.ValidationError, "exactly once"):
            protocol.normalize_historical_prompt(b"no absolute output path")


class CacheEnvironmentTests(unittest.TestCase):
    def test_expected_xdg_packages_path_accepts_absolute_xdg_without_home(self) -> None:
        with mock.patch.dict(
            os.environ,
            {"XDG_CACHE_HOME": "/var/tmp/harp-cache"},
            clear=True,
        ):
            self.assertEqual(
                protocol.expected_xdg_packages_path(),
                Path("/var/tmp/harp-cache/harp/lean/lean-4.32.1/packages"),
            )

    def test_expected_xdg_packages_path_rejects_relative_home_fallback(self) -> None:
        with mock.patch.dict(os.environ, {"HOME": "relative-home"}, clear=True):
            with self.assertRaisesRegex(protocol.ValidationError, "HOME must be absolute"):
                protocol.expected_xdg_packages_path()


class RunSpecTests(unittest.TestCase):
    def test_run_spec_accepts_closed_historical_contract(self) -> None:
        value = run_spec()

        self.assertEqual(protocol.validate_run_spec(value), value)

    def test_run_spec_rejects_unknown_fields_invalid_enums_and_bounds(self) -> None:
        value = run_spec()
        value["surprise"] = True
        with self.assertRaisesRegex(protocol.ValidationError, "fields"):
            protocol.validate_run_spec(value)

        value = run_spec()
        value["arm"] = "replay"
        with self.assertRaisesRegex(protocol.ValidationError, "arm"):
            protocol.validate_run_spec(value)

        value = run_spec()
        value["max_calls"] = 0
        with self.assertRaisesRegex(protocol.ValidationError, "max_calls"):
            protocol.validate_run_spec(value)

        value = run_spec()
        value["generation_visible_files"] = ["../proof.tex"]
        with self.assertRaisesRegex(protocol.ValidationError, "relative"):
            protocol.validate_run_spec(value)

    def test_strict_json_reader_rejects_symlinks_unknown_shape_and_oversize(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "run_spec.json"
            path.write_text(json.dumps(run_spec()))
            self.assertEqual(protocol.read_run_spec(path), run_spec())

            link = root / "link.json"
            link.symlink_to(path)
            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                protocol.read_run_spec(link)

            path.write_bytes(b"x" * (protocol.MAX_JSON_BYTES + 1))
            with self.assertRaisesRegex(protocol.ValidationError, "byte cap"):
                protocol.read_run_spec(path)


class RouteRegistryTests(unittest.TestCase):
    def test_valid_route_can_block_reopen_with_new_mechanism_and_promote(self) -> None:
        registry = protocol.RouteRegistry("orchestrated-001")
        registry.apply(route_event(1, "route-1", None, "independent"))
        registry.apply(route_event(2, "route-1", "independent", "blocked"))
        registry.apply(
            route_event(
                3,
                "route-1",
                "blocked",
                "independent",
                mechanism="positive kernel with exact cancellation",
            )
        )
        registry.apply(
            route_event(
                4,
                "route-1",
                "independent",
                "viable",
                mechanism="positive kernel with exact cancellation",
                concrete_artifacts=[
                    "positive kernel",
                    "cancellation identity",
                    "constant-two endpoint",
                ],
            )
        )
        registry.apply(
            route_event(
                5,
                "route-1",
                "viable",
                "audited",
                mechanism="positive kernel with exact cancellation",
                concrete_artifacts=["standalone candidate"],
                candidate_sha256="d" * 64,
            )
        )
        registry.apply(
            route_event(
                6,
                "route-1",
                "audited",
                "promoted",
                mechanism="positive kernel with exact cancellation",
                concrete_artifacts=["standalone candidate"],
                candidate_sha256="d" * 64,
            )
        )

        self.assertEqual(registry.states, {"route-1": "promoted"})
        self.assertEqual(len(registry.events), 6)

    def test_registry_rejects_duplicate_sequence_invalid_transition_and_reopen(self) -> None:
        registry = protocol.RouteRegistry("orchestrated-001")
        registry.apply(route_event(1, "route-1", None, "independent"))
        with self.assertRaisesRegex(protocol.ValidationError, "sequence"):
            registry.apply(route_event(1, "route-2", None, "independent"))
        with self.assertRaisesRegex(protocol.ValidationError, "transition"):
            registry.apply(route_event(2, "route-1", "independent", "promoted"))

        registry.apply(route_event(2, "route-1", "independent", "blocked"))
        with self.assertRaisesRegex(protocol.ValidationError, "new mechanism"):
            registry.apply(route_event(3, "route-1", "blocked", "independent"))

    def test_viability_audit_and_promotion_fail_closed(self) -> None:
        registry = protocol.RouteRegistry("orchestrated-001")
        registry.apply(route_event(1, "route-1", None, "independent"))
        with self.assertRaisesRegex(protocol.ValidationError, "concrete"):
            registry.apply(route_event(2, "route-1", "independent", "viable"))

        registry.apply(
            route_event(
                2,
                "route-1",
                "independent",
                "viable",
                concrete_artifacts=["recurrence"],
            )
        )
        with self.assertRaisesRegex(protocol.ValidationError, "candidate"):
            registry.apply(route_event(3, "route-1", "viable", "audited"))

        registry.apply(
            route_event(
                3,
                "route-1",
                "viable",
                "audited",
                concrete_artifacts=["candidate"],
                candidate_sha256="d" * 64,
                unresolved_critical_findings=1,
            )
        )
        with self.assertRaisesRegex(protocol.ValidationError, "critical"):
            registry.apply(
                route_event(
                    4,
                    "route-1",
                    "audited",
                    "promoted",
                    candidate_sha256="d" * 64,
                    unresolved_critical_findings=1,
                )
            )

        registry = protocol.RouteRegistry("orchestrated-001")
        registry.apply(route_event(1, "route-2", None, "independent"))
        registry.apply(
            route_event(
                2,
                "route-2",
                "independent",
                "viable",
                concrete_artifacts=["candidate"],
            )
        )
        registry.apply(
            route_event(
                3,
                "route-2",
                "viable",
                "audited",
                candidate_sha256="d" * 64,
                unproved_obligations=[
                    {
                        "statement": "prove the conjecture for all matrices",
                        "strength": "theorem_strength",
                    }
                ],
            )
        )
        with self.assertRaisesRegex(protocol.ValidationError, "theorem-strength"):
            registry.apply(
                route_event(
                    4,
                    "route-2",
                    "audited",
                    "promoted",
                    candidate_sha256="d" * 64,
                    unproved_obligations=[
                        {
                            "statement": "prove the conjecture for all matrices",
                            "strength": "theorem_strength",
                        }
                    ],
                )
            )


class RunPreparationTests(unittest.TestCase):
    def test_prepare_historical_run_binds_prompt_cli_and_authored_inputs(self) -> None:
        source = (
            b"theorem bytes\nCurrent task statement\nwrite to "
            + protocol.HISTORICAL_OUTPUT_PATH.encode()
            + b"\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prompt_path = root / "historical.txt"
            prompt_path.write_bytes(source)
            cli_path = root / "traecli"
            cli_path.write_text("#!/bin/sh\nprintf '%s\\n' 'traecli test-version'\n")
            cli_path.chmod(0o755)
            run_dir = root / "run"
            with mock.patch.object(
                prepare_run,
                "verify_historical_prompt",
                side_effect=lambda data: protocol.verify_historical_prompt(
                    data,
                    expected_bytes=len(source),
                    expected_sha256=digest(source),
                ),
            ):
                result = prepare_run.prepare_run(
                    run_dir=run_dir,
                    arm="historical",
                    historical_prompt_path=prompt_path,
                    cli_path=cli_path,
                    model="gpt-5.6-sol",
                    timeout_seconds=600,
                    created_at_utc="2026-08-15T04:00:00Z",
                )

            self.assertEqual(result, run_dir.resolve())
            spec = protocol.read_run_spec(run_dir / "run_spec.json")
            self.assertEqual(spec["arm"], "historical")
            self.assertEqual(spec["leakage"], "L1")
            self.assertEqual(spec["max_calls"], 1)
            self.assertEqual(spec["allowed_tools"], ["Write", "spawn_agent"])
            self.assertEqual(spec["cli"]["version"], "traecli test-version")
            self.assertEqual(spec["cli"]["sha256"], digest(cli_path.read_bytes()))
            self.assertEqual(
                (run_dir / "inputs/theorem.txt").read_bytes(),
                b"theorem bytes\n",
            )
            execution = (run_dir / "inputs/execution_prompt.txt").read_bytes()
            self.assertNotIn(protocol.HISTORICAL_OUTPUT_PATH.encode(), execution)
            self.assertIn(b"candidate.tex", execution)
            self.assertFalse((run_dir / "inputs/historical_prompt.txt").exists())
            normalization = json.loads(
                (run_dir / "inputs/prompt_normalization.json").read_text()
            )
            self.assertEqual(normalization["source_sha256"], digest(source))
            self.assertEqual(
                sorted(path.name for path in (run_dir / "schemas").iterdir()),
                ["historical_final.schema.json"],
            )
            self.assertTrue((run_dir / "calls").is_dir())
            self.assertTrue((run_dir / "candidate").is_dir())
            self.assertTrue((run_dir / "review").is_dir())

            with self.assertRaisesRegex(protocol.ValidationError, "existing"):
                prepare_run.prepare_run(
                    run_dir=run_dir,
                    arm="historical",
                    historical_prompt_path=prompt_path,
                    cli_path=cli_path,
                    model="gpt-5.6-sol",
                    timeout_seconds=600,
                    created_at_utc="2026-08-15T04:00:00Z",
                )

    def test_prepare_orchestrated_run_copies_only_blind_templates(self) -> None:
        source = (
            b"theorem bytes\nCurrent task statement\nwrite to "
            + protocol.HISTORICAL_OUTPUT_PATH.encode()
            + b"\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prompt_path = root / "historical.txt"
            prompt_path.write_bytes(source)
            cli_path = root / "traecli"
            cli_path.write_text("#!/bin/sh\nprintf '%s\\n' 'traecli test-version'\n")
            cli_path.chmod(0o755)
            run_dir = root / "run"
            with mock.patch.object(
                prepare_run,
                "verify_historical_prompt",
                side_effect=lambda data: protocol.verify_historical_prompt(
                    data,
                    expected_bytes=len(source),
                    expected_sha256=digest(source),
                ),
            ):
                prepare_run.prepare_run(
                    run_dir=run_dir,
                    arm="orchestrated",
                    historical_prompt_path=prompt_path,
                    cli_path=cli_path,
                    model="gpt-5.6-sol",
                    timeout_seconds=600,
                    created_at_utc="2026-08-15T04:00:00Z",
                )

            spec = protocol.read_run_spec(run_dir / "run_spec.json")
            self.assertEqual(spec["leakage"], "L1")
            self.assertEqual(spec["max_calls"], 9)
            self.assertEqual(spec["allowed_tools"], ["Write"])
            prompt_names = sorted(
                path.name for path in (run_dir / "prompts").iterdir()
            )
            self.assertEqual(
                prompt_names,
                [
                    "controller.md",
                    "critic.md",
                    "redirect.md",
                    "repair.md",
                    "route_worker.md",
                    "synthesizer.md",
                ],
            )
            combined = "\n".join(
                path.read_text() for path in (run_dir / "prompts").iterdir()
            ).lower()
            for forbidden in [
                "herglotz",
                "origin sample",
                "conjugate eigen",
                "lorist",
                "schwenninger",
                "e_n",
            ]:
                self.assertNotIn(forbidden, combined)
            self.assertNotIn("crouzeix_conjecture", combined)
            self.assertNotIn(str(Path.cwd()), combined)

    def test_extract_theorem_requires_one_task_marker(self) -> None:
        self.assertEqual(
            prepare_run.extract_theorem(b"A\nCurrent task statement\nB\n"),
            b"A\n",
        )
        for malformed in [
            b"no marker",
            b"A\nCurrent task statement\nB\nCurrent task statement\nC\n",
        ]:
            with self.assertRaisesRegex(protocol.ValidationError, "marker"):
                prepare_run.extract_theorem(malformed)

    def test_installed_cli_resolves_from_path_and_rejects_absence(self) -> None:
        with mock.patch.object(
            prepare_run.shutil,
            "which",
            return_value="/tmp/tools/traecli",
        ):
            self.assertEqual(
                prepare_run._installed_cli(),
                Path("/tmp/tools/traecli").resolve(),
            )
        with mock.patch.object(prepare_run.shutil, "which", return_value=None):
            with self.assertRaisesRegex(protocol.ValidationError, "not installed"):
                prepare_run._installed_cli()


if __name__ == "__main__":
    unittest.main()
