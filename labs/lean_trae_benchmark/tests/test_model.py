from __future__ import annotations

import dataclasses
import sys
import tempfile
import unittest
from pathlib import Path


sys.path.insert(0, str(Path(__file__).parents[1]))

from model import (
    ArtifactRef,
    LedgerEvent,
    RuntimeLock,
    TaskSeal,
    ValidationError,
    canonical_json_bytes,
    parse_json_object,
    sha256_bytes,
)


def digest(hex_value: str = "a" * 64) -> dict[str, str]:
    return {"algorithm": "sha256", "hex": hex_value}


def artifact(path: str, *, bytes: int = 1, hex_value: str = "a" * 64) -> dict[str, object]:
    return {"path": path, "bytes": bytes, "digest": digest(hex_value)}


def runtime_lock() -> dict[str, object]:
    return {
        "schema_version": "harp-lean-runtime-lock/v1",
        "runtime_id": "lean-trae-test-v1",
        "platform": {"os": "darwin", "arch": "arm64"},
        "trae": {
            "version": "0.200.19",
            "artifact": {
                "source_url": "https://example.test/traecli",
                "digest": digest(),
                "bytes": 1,
            },
            "published_path": "runtime/darwin-arm64/traecli",
            "version_stdout": "traecli 0.200.19",
        },
        "lean": {
            "version": "4.19.0",
            "toolchain_id": "leanprover/lean4:v4.19.0",
            "artifact_digest": digest("b" * 64),
            "elan_layout_digest": digest("c" * 64),
        },
        "mathlib": None,
    }


def task_seal() -> dict[str, object]:
    return {
        "schema_version": "harp-lean-task-seal/v1",
        "task_id": "core-and-left",
        "profile": "core-proposition",
        "runtime_id": "lean-trae-test-v1",
        "generation_budget": 1,
        "repair_budget": 2,
        "candidate_encoding": "utf-8-proof-body",
        "slots": [
            {
                "slot_id": "target",
                "source_path": "Task.lean",
                "declaration_name": "core_and_left",
                "declaration_type": "forall p q, p and q implies p",
                "prefix": artifact("sealed/Task.prefix.lean"),
                "suffix": artifact("sealed/Task.suffix.lean"),
                "mode": "proof_body",
                "forbidden_tokens": ["import", "sorry"],
            }
        ],
        "project_inventory": [artifact("lakefile.toml")],
        "command_policy": {
            "commands": [{"argv": ["lean", "Candidate.lean"], "cwd": "."}],
            "writable_paths": ["Scratch.lean", "Candidate.lean"],
            "network": False,
        },
        "prompt": artifact("prompt.md"),
    }


class ModelTests(unittest.TestCase):
    def test_canonical_json_rejects_duplicate_keys_and_hashes_exact_utf8_bytes(self) -> None:
        with self.assertRaisesRegex(ValidationError, "duplicate key"):
            parse_json_object(b'{"a":1,"a":2}', "fixture")
        self.assertEqual(
            sha256_bytes(canonical_json_bytes({"b": 2, "a": 1})).hex,
            sha256_bytes(b'{"a":1,"b":2}').hex,
        )

    def test_artifact_ref_rejects_absolute_traversal_symlink_and_wrong_digest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            payload = root / "sealed.txt"
            payload.write_bytes(b"sealed bytes")
            reference = ArtifactRef.from_file(root, "sealed.txt")
            reference.verify(root)

            for bad_path in ("/sealed.txt", "../sealed.txt", "nested/../../sealed.txt"):
                with self.assertRaisesRegex(ValidationError, "relative"):
                    ArtifactRef.from_value(artifact(bad_path), "artifact")

            link = root / "linked.txt"
            link.symlink_to(payload)
            linked = ArtifactRef.from_value(
                artifact("linked.txt", bytes=len(b"sealed bytes"), hex_value=reference.digest.hex),
                "artifact",
            )
            with self.assertRaisesRegex(ValidationError, "symlink"):
                linked.verify(root)

            wrong = dataclasses.replace(reference, digest=sha256_bytes(b"different"))
            with self.assertRaisesRegex(ValidationError, "digest"):
                wrong.verify(root)

    def test_runtime_lock_accepts_exact_schema_and_rejects_unknown_or_invalid_enums(self) -> None:
        value = runtime_lock()
        lock = RuntimeLock.from_value(value, "runtime lock")
        self.assertEqual(lock.trae.version, "0.200.19")

        unknown = dict(value)
        unknown["extra"] = True
        with self.assertRaisesRegex(ValidationError, "fields are not exact"):
            RuntimeLock.from_value(unknown, "runtime lock")
        bad_platform = runtime_lock()
        bad_platform["platform"] = {"os": "plan9", "arch": "arm64"}
        with self.assertRaisesRegex(ValidationError, "platform os"):
            RuntimeLock.from_value(bad_platform, "runtime lock")

    def test_task_seal_accepts_closed_single_slot_schema_and_rejects_invalid_profile(self) -> None:
        value = task_seal()
        seal = TaskSeal.from_value(value, "task seal")
        self.assertEqual(seal.slots[0].slot_id, "target")
        self.assertIsNone(seal.slot_order)

        bad = task_seal()
        bad["profile"] = "unsealed"
        with self.assertRaisesRegex(ValidationError, "profile"):
            TaskSeal.from_value(bad, "task seal")
        bad = task_seal()
        bad["command_policy"]["network"] = "false"
        with self.assertRaisesRegex(ValidationError, "network"):
            TaskSeal.from_value(bad, "task seal")

    def test_task_seal_requires_the_profile_policy_and_bounded_budgets(self) -> None:
        for mutate, message in (
            (lambda value: value.__setitem__("generation_budget", 0), "generation_budget"),
            (lambda value: value.__setitem__("repair_budget", 3), "repair_budget"),
            (lambda value: value["command_policy"].__setitem__("network", True), "network"),
            (
                lambda value: value["command_policy"].__setitem__(
                    "commands", [{"argv": ["curl", "https://example.test"], "cwd": "."}]
                ),
                "profile policy",
            ),
        ):
            value = task_seal()
            mutate(value)
            with self.assertRaisesRegex(ValidationError, message):
                TaskSeal.from_value(value, "task seal")

        structural = task_seal()
        structural["profile"] = "core-structural"
        self.assertEqual(TaskSeal.from_value(structural, "task seal").profile, "core-structural")

        mathlib = task_seal()
        mathlib["profile"] = "mathlib-add-comm"
        mathlib["command_policy"]["commands"] = [
            {"argv": ["lake", "env", "lean", "Candidate.lean"], "cwd": "."}
        ]
        self.assertEqual(TaskSeal.from_value(mathlib, "task seal").profile, "mathlib-add-comm")

    def test_ledger_event_deep_freezes_payload_containers(self) -> None:
        event = LedgerEvent.from_value(
            {
                "schema_version": "harp-lean-ledger-event/v1",
                "sequence": 0,
                "run_id": "run-1",
                "attempt": 0,
                "kind": "runtime_verified",
                "previous_event_digest": None,
                "payload": {"nested": {"values": ["sealed"]}},
                "event_digest": digest(),
            }
        )

        with self.assertRaises(TypeError):
            event.payload["other"] = "mutation"  # type: ignore[index]
        nested = event.payload["nested"]
        self.assertIsInstance(nested, type(event.payload))
        with self.assertRaises(TypeError):
            nested["other"] = "mutation"  # type: ignore[index]
        self.assertEqual(event.payload["nested"]["values"], ("sealed",))
        self.assertEqual(
            canonical_json_bytes(event.payload),
            b'{"nested":{"values":["sealed"]}}',
        )
