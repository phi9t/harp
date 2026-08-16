from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import formal_receipt
import protocol
import tickets


def digest(data: bytes) -> str:
    return protocol.sha256_bytes(data)


def formal_ticket(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-runtime-ticket/v1",
        "ticket_id": "formal-node-g1-d0",
        "run_id": "expert-frontier-001",
        "task_kind": "formal_attempt",
        "node_id": "node-g1-d0",
        "parent_node_id": None,
        "generation": 1,
        "direction_id": "formalize-candidate",
        "role": "formal_verifier",
        "objective": "Validate the frozen Crouzeix candidate against fake Lean fixtures.",
        "expected_deliverable": "Formal attempt receipt with typed terminal status.",
        "dependency_ticket_ids": ["review-candidate-a"],
        "context_sha256": "a" * 64,
        "schema_sha256": "b" * 64,
        "prompt_sha256": "c" * 64,
        "parent_artifact_sha256": "d" * 64,
        "allowed_tools": ["Read"],
        "forbidden_sources": ["public proof manuscripts"],
        "timeout_seconds": 3600,
        "max_output_bytes": 1048576,
        "owner_type": "orchestrator",
        "created_at_utc": "2026-08-15T12:00:00Z",
        "state": "created",
    }
    value.update(overrides)
    return value


def request(**overrides: object) -> dict[str, object]:
    candidate = b"candidate proof bytes\n"
    source = b"theorem CrouzeixFake : True := by trivial\n"
    command = ["lake", "env", "lean", "CrouzeixFake.lean"]
    value: dict[str, object] = {
        "attempt_id": "formal-attempt-001",
        "run_id": "expert-frontier-001",
        "candidate": {
            "outcome": "candidate",
            "candidate_id": "candidate-a",
            "candidate_sha256": digest(candidate),
            "candidate_bytes": len(candidate),
        },
        "toolchain": {
            "name": "lean",
            "version": "4.12.0",
            "platform": "fake-darwin-arm64",
            "toolchain_sha256": "1" * 64,
        },
        "source": {
            "path": "source/CrouzeixFake.lean",
            "sha256": digest(source),
            "bytes": len(source),
        },
        "command": {
            "argv": command,
            "cwd": ".",
            "env_sha256": "2" * 64,
            "exit_code": 0,
        },
        "logs": {
            "stdout_path": "logs/stdout.txt",
            "stdout_sha256": digest(b"ok\n"),
            "stderr_path": "logs/stderr.txt",
            "stderr_sha256": digest(b""),
            "complete": True,
        },
        "resource_receipt": {
            "path": "resource_receipt.json",
            "sha256": "",
        },
        "axioms": {
            "scan_performed": True,
            "scanner": "fake-lean-axiom-scan",
            "allowed_axioms": [],
            "observed_axioms": [],
            "scan_log_path": "logs/axioms.txt",
            "scan_log_sha256": digest(b"#print axioms output\n"),
        },
        "status": "passed",
        "reason": "fake Lean command succeeded and axiom scan reported no axioms",
        "started_at_utc": "2026-08-15T12:00:01Z",
        "completed_at_utc": "2026-08-15T12:00:02Z",
    }
    value.update(overrides)
    return value


def resource_receipt(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-formal-resource-receipt/v1",
        "host_space_bytes": 8 * 1024 * 1024 * 1024,
        "memory_limit_bytes": 4 * 1024 * 1024 * 1024,
        "cpu_limit": 4,
        "preflight_status": "ok",
        "checked_at_utc": "2026-08-15T12:00:00Z",
    }
    value.update(overrides)
    return value


def receipt_value(**overrides: object) -> dict[str, object]:
    value = request()
    value["schema_version"] = "crouzeix-formal-attempt-receipt/v1"
    value["ticket_id"] = "formal-node-g1-d0"
    value["ticket_sha256"] = tickets.canonical_sha256(formal_ticket())
    value["resource_receipt"] = {
        "path": "resource_receipt.json",
        "sha256": "3" * 64,
    }
    value.update(overrides)
    value["formal_attempt_sha256"] = formal_receipt.canonical_sha256_without_self(value)
    return value


def v2_receipt_value(**overrides: object) -> dict[str, object]:
    value = receipt_value()
    value["schema_version"] = "crouzeix-formal-attempt-receipt/v2"
    value["formal_target"] = {
        "path": "formal_target.lock.json",
        "sha256": "4" * 64,
    }
    value["runtime_inventory_sha256"] = "5" * 64
    value["target_type_sha256"] = "6" * 64
    value.update(overrides)
    value["formal_attempt_sha256"] = formal_receipt.canonical_sha256_without_self(value)
    return value


class FormalReceiptTests(unittest.TestCase):
    def test_prepare_attempt_is_create_only_and_validates_strict_receipt_fields(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            ticket = formal_ticket()
            resource = resource_receipt()
            payload = request()
            payload["resource_receipt"] = {
                "path": "resource_receipt.json",
                "sha256": formal_receipt.canonical_sha256(resource),
            }

            attempt_dir = formal_receipt.prepare_attempt(
                root / "formal-attempt-001",
                ticket=ticket,
                request=payload,
                source_bytes=b"theorem CrouzeixFake : True := by trivial\n",
                stdout_bytes=b"ok\n",
                stderr_bytes=b"",
                axiom_log_bytes=b"#print axioms output\n",
                resource_receipt=resource,
            )

            receipt = formal_receipt.validate_attempt_dir(attempt_dir)
            self.assertEqual(receipt["status"], "passed")
            self.assertEqual(receipt["ticket_sha256"], tickets.canonical_sha256(ticket))
            self.assertEqual(receipt["formal_attempt_sha256"], formal_receipt.canonical_sha256_without_self(receipt))
            self.assertEqual(
                sorted(path.name for path in attempt_dir.iterdir()),
                ["attempt.json", "logs", "receipt.json", "resource_receipt.json", "source"],
            )

            bad = dict(receipt)
            bad["unknown"] = True
            with self.assertRaisesRegex(protocol.ValidationError, "fields"):
                formal_receipt.validate_receipt(bad)

            with self.assertRaisesRegex(protocol.ValidationError, "already exists"):
                formal_receipt.prepare_attempt(
                    attempt_dir,
                    ticket=ticket,
                    request=payload,
                    source_bytes=b"theorem CrouzeixFake : True := by trivial\n",
                    stdout_bytes=b"ok\n",
                    stderr_bytes=b"",
                    axiom_log_bytes=b"#print axioms output\n",
                    resource_receipt=resource,
                )

    def test_prepare_rejects_existing_symlink_and_unsafe_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            real = root / "real"
            real.mkdir()
            link = root / "link"
            link.symlink_to(real)
            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                formal_receipt.prepare_attempt(
                    link,
                    ticket=formal_ticket(),
                    request=request(),
                    source_bytes=b"",
                    stdout_bytes=b"",
                    stderr_bytes=b"",
                    axiom_log_bytes=b"",
                    resource_receipt=resource_receipt(),
                )

            bad_request = request(source={"path": "../escape.lean", "sha256": "0" * 64, "bytes": 0})
            with self.assertRaisesRegex(protocol.ValidationError, "relative"):
                formal_receipt.prepare_attempt(
                    root / "unsafe",
                    ticket=formal_ticket(),
                    request=bad_request,
                    source_bytes=b"",
                    stdout_bytes=b"",
                    stderr_bytes=b"",
                    axiom_log_bytes=b"",
                    resource_receipt=resource_receipt(),
                )

    def test_validate_attempt_dir_recomputes_digests_and_rejects_tampering(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            resource = resource_receipt()
            payload = request()
            payload["resource_receipt"] = {
                "path": "resource_receipt.json",
                "sha256": formal_receipt.canonical_sha256(resource),
            }
            attempt_dir = formal_receipt.prepare_attempt(
                root / "formal-attempt-001",
                ticket=formal_ticket(),
                request=payload,
                source_bytes=b"theorem CrouzeixFake : True := by trivial\n",
                stdout_bytes=b"ok\n",
                stderr_bytes=b"",
                axiom_log_bytes=b"#print axioms output\n",
                resource_receipt=resource,
            )

            (attempt_dir / "source/CrouzeixFake.lean").write_bytes(b"tampered\n")
            with self.assertRaisesRegex(protocol.ValidationError, "source.sha256"):
                formal_receipt.validate_attempt_dir(attempt_dir)

    def test_validate_attempt_dir_binds_resource_receipt_path_not_default_name(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            resource = resource_receipt()
            payload = request()
            payload["resource_receipt"] = {
                "path": "resource_receipt.json",
                "sha256": formal_receipt.canonical_sha256(resource),
            }
            attempt_dir = formal_receipt.prepare_attempt(
                root / "formal-attempt-001",
                ticket=formal_ticket(),
                request=payload,
                source_bytes=b"theorem CrouzeixFake : True := by trivial\n",
                stdout_bytes=b"ok\n",
                stderr_bytes=b"",
                axiom_log_bytes=b"#print axioms output\n",
                resource_receipt=resource,
            )
            receipt_path = attempt_dir / "receipt.json"
            receipt = json.loads(receipt_path.read_text())
            receipt["resource_receipt"]["path"] = "other-resource.json"
            receipt["formal_attempt_sha256"] = formal_receipt.canonical_sha256_without_self(receipt)
            receipt_path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")

            with self.assertRaisesRegex(protocol.ValidationError, "resource_receipt.path"):
                formal_receipt.validate_attempt_dir(attempt_dir)

    def test_validate_attempt_dir_rejects_symlinked_artifact_ancestors(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            resource = resource_receipt()
            payload = request()
            payload["resource_receipt"] = {
                "path": "resource_receipt.json",
                "sha256": formal_receipt.canonical_sha256(resource),
            }
            attempt_dir = formal_receipt.prepare_attempt(
                root / "formal-attempt-001",
                ticket=formal_ticket(),
                request=payload,
                source_bytes=b"theorem CrouzeixFake : True := by trivial\n",
                stdout_bytes=b"ok\n",
                stderr_bytes=b"",
                axiom_log_bytes=b"#print axioms output\n",
                resource_receipt=resource,
            )
            source_target = root / "source-target"
            source_target.mkdir()
            (source_target / "CrouzeixFake.lean").write_bytes(
                b"theorem CrouzeixFake : True := by trivial\n"
            )
            for path in sorted((attempt_dir / "source").rglob("*"), reverse=True):
                path.unlink()
            (attempt_dir / "source").rmdir()
            (attempt_dir / "source").symlink_to(source_target)

            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                formal_receipt.validate_attempt_dir(attempt_dir)

    def test_blocked_status_requires_resource_block_not_ordinary_command_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            resource = resource_receipt(preflight_status="ok")
            payload = request(
                status="blocked",
                command=request()["command"] | {"exit_code": 1},
                reason="ordinary fake Lean command failure",
            )
            payload["resource_receipt"] = {
                "path": "resource_receipt.json",
                "sha256": formal_receipt.canonical_sha256(resource),
            }
            attempt_dir = formal_receipt.prepare_attempt(
                root / "formal-attempt-001",
                ticket=formal_ticket(),
                request=payload,
                source_bytes=b"theorem CrouzeixFake : True := by trivial\n",
                stdout_bytes=b"ok\n",
                stderr_bytes=b"",
                axiom_log_bytes=b"#print axioms output\n",
                resource_receipt=resource,
            )

            with self.assertRaisesRegex(protocol.ValidationError, "blocked"):
                formal_receipt.validate_attempt_dir(attempt_dir)

    def test_terminal_status_invariants_keep_passed_failed_and_blocked_distinct(self) -> None:
        base = receipt_value()

        with self.assertRaisesRegex(protocol.ValidationError, "passed"):
            formal_receipt.validate_receipt(
                receipt_value(command=base["command"] | {"exit_code": 1})
            )
        with self.assertRaisesRegex(protocol.ValidationError, "complete logs"):
            formal_receipt.validate_receipt(
                receipt_value(logs=base["logs"] | {"complete": False})
            )
        with self.assertRaisesRegex(protocol.ValidationError, "axiom scan"):
            formal_receipt.validate_receipt(
                receipt_value(axioms=base["axioms"] | {"scan_performed": False})
            )

        failed = receipt_value(status="failed", command=base["command"] | {"exit_code": 1})
        self.assertEqual(formal_receipt.validate_receipt(failed)["status"], "failed")

        blocked = receipt_value(
            status="blocked",
            command=base["command"] | {"exit_code": None},
            resource_receipt={
                "path": "resource_receipt.json",
                "sha256": "3" * 64,
            },
        )
        self.assertEqual(formal_receipt.validate_receipt(blocked)["status"], "blocked")
        with self.assertRaisesRegex(protocol.ValidationError, "not_attempted"):
            formal_receipt.validate_receipt(
                receipt_value(
                    status="not_attempted",
                    command=base["command"] | {"exit_code": 0},
                )
            )

    def test_cli_validate_checks_fake_attempt_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            resource = resource_receipt()
            payload = request()
            payload["resource_receipt"] = {
                "path": "resource_receipt.json",
                "sha256": formal_receipt.canonical_sha256(resource),
            }
            attempt_dir = formal_receipt.prepare_attempt(
                root / "formal-attempt-001",
                ticket=formal_ticket(),
                request=payload,
                source_bytes=b"theorem CrouzeixFake : True := by trivial\n",
                stdout_bytes=b"ok\n",
                stderr_bytes=b"",
                axiom_log_bytes=b"#print axioms output\n",
                resource_receipt=resource,
            )

            completed = subprocess.run(
                [
                    sys.executable,
                    str(LAB / "formal_receipt.py"),
                    "validate",
                    "--attempt-dir",
                    str(attempt_dir),
                ],
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(completed.returncode, 0, completed.stderr)
            self.assertEqual(json.loads(completed.stdout)["status"], "passed")

    def test_v2_receipt_binds_formal_target_runtime_inventory_and_target_type(self) -> None:
        receipt = formal_receipt.validate_receipt(v2_receipt_value())

        self.assertEqual(receipt["schema_version"], "crouzeix-formal-attempt-receipt/v2")
        self.assertEqual(receipt["formal_target"]["path"], "formal_target.lock.json")
        self.assertEqual(receipt["runtime_inventory_sha256"], "5" * 64)
        self.assertEqual(receipt["target_type_sha256"], "6" * 64)

    def test_v1_rejects_v2_fields_and_v2_requires_target_binding(self) -> None:
        with self.assertRaisesRegex(protocol.ValidationError, "unknown"):
            formal_receipt.validate_receipt(
                receipt_value(formal_target={"path": "formal_target.lock.json", "sha256": "4" * 64})
            )

        missing = v2_receipt_value()
        missing.pop("runtime_inventory_sha256")
        with self.assertRaisesRegex(protocol.ValidationError, "runtime_inventory_sha256"):
            formal_receipt.validate_receipt(missing)

    def test_v2_rejects_target_drift_and_forbidden_axioms(self) -> None:
        with self.assertRaisesRegex(protocol.ValidationError, "formal_target.sha256"):
            formal_receipt.validate_receipt(
                v2_receipt_value(formal_target={"path": "formal_target.lock.json", "sha256": "bad"})
            )

        base_axioms = request()["axioms"]
        with self.assertRaisesRegex(protocol.ValidationError, "disallowed axioms"):
            formal_receipt.validate_receipt(
                v2_receipt_value(
                    axioms=base_axioms | {"allowed_axioms": [], "observed_axioms": ["Classical.choice"]}
                )
            )


if __name__ == "__main__":
    unittest.main()
