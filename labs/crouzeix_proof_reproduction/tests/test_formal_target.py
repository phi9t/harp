from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import formal_target
import protocol


def digest(data: bytes) -> str:
    return protocol.sha256_bytes(data)


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def row(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-formal-ledger-row/v1",
        "row_id": "lemma-foundation",
        "statement_sha256": "a" * 64,
        "source_locator": "Harp-authored",
        "dependency_ids": [],
        "owner_route": "jin",
        "status": "locally_proved",
    }
    value.update(overrides)
    return value


def lock_for(root: Path, *, artifact_path: str = "artifacts/Target.lean") -> dict[str, object]:
    artifact = root / artifact_path
    return {
        "schema_version": "crouzeix-formal-target-lock/v1",
        "target": {
            "target_id": "crouzeix-main",
            "declaration_name": "crouzeixConjecture",
            "statement_sha256": "b" * 64,
            "source_locator": "Harp-authored",
            "dependency_ids": ["lemma-foundation"],
        },
        "artifacts": [
            {
                "artifact_id": "target-lean",
                "path": artifact_path,
                "bytes": artifact.stat().st_size if artifact.exists() else 0,
                "sha256": digest(artifact.read_bytes()) if artifact.exists() else "0" * 64,
            }
        ],
        "ledger_path": "ledger",
    }


class FormalTargetTests(unittest.TestCase):
    def make_root(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        directory = tempfile.TemporaryDirectory()
        root = Path(directory.name).resolve()
        (root / "artifacts").mkdir()
        (root / "ledger").mkdir()
        return directory, root

    def write_valid_target(self, root: Path) -> dict[str, object]:
        (root / "artifacts" / "Target.lean").write_bytes(b"theorem crouzeixConjecture : True := by trivial\n")
        formal_target.append_ledger_row(
            root / "ledger",
            formal_target.LedgerRow.from_mapping(row()),
        )
        return lock_for(root)

    def test_production_loader_ignores_ambient_redirect_and_uses_production_path(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            production_lock = root / "production.lock.json"
            redirect_lock = root / "redirect.lock.json"
            (root / "artifacts").mkdir()
            (root / "ledger").mkdir()
            (root / "artifacts" / "Target.lean").write_bytes(b"target\n")
            write_json(production_lock, lock_for(root))
            write_json(redirect_lock, {"schema_version": "evil", "unknown": True})

            with mock.patch.object(formal_target, "PRODUCTION_LOCK_PATH", production_lock):
                with mock.patch.dict(os.environ, {"CROUZEIX_FORMAL_TARGET_LOCK": str(redirect_lock)}):
                    loaded = formal_target.load_lock()

            self.assertEqual(loaded.target.target_id, "crouzeix-main")

    def test_test_only_loader_accepts_explicit_temporary_lock(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            (root / "artifacts").mkdir()
            (root / "ledger").mkdir()
            (root / "artifacts" / "Target.lean").write_bytes(b"target\n")
            path = root / "explicit.lock.json"
            write_json(path, lock_for(root))

            loaded = formal_target.load_lock_for_test(path)

            self.assertEqual(loaded.artifacts[0].path, "artifacts/Target.lean")

    def test_duplicate_json_keys_and_unknown_fields_fail(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            duplicate = root / "duplicate.lock.json"
            duplicate.write_text(
                '{"schema_version":"crouzeix-formal-target-lock/v1","schema_version":"again"}',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(protocol.ValidationError, "duplicate"):
                formal_target.load_lock_for_test(duplicate)

            unknown = root / "unknown.lock.json"
            write_json(
                unknown,
                {
                    "schema_version": "crouzeix-formal-target-lock/v1",
                    "target": {
                        "target_id": "crouzeix-main",
                        "declaration_name": "crouzeixConjecture",
                        "statement_sha256": "b" * 64,
                        "source_locator": "Harp-authored",
                        "dependency_ids": [],
                        "extra": True,
                    },
                    "artifacts": [],
                    "ledger_path": "ledger",
                },
            )
            with self.assertRaisesRegex(protocol.ValidationError, "unknown"):
                formal_target.load_lock_for_test(unknown)

    def test_absolute_traversal_and_symlinked_artifact_paths_fail(self) -> None:
        directory, root = self.make_root()
        with directory:
            (root / "artifacts" / "Target.lean").write_bytes(b"target\n")
            formal_target.append_ledger_row(root / "ledger", formal_target.LedgerRow.from_mapping(row()))

            for bad_path in ("/tmp/Target.lean", "../Target.lean"):
                with self.subTest(path=bad_path):
                    with self.assertRaisesRegex(protocol.ValidationError, "relative"):
                        formal_target.FormalTargetLock.from_mapping(lock_for(root, artifact_path=bad_path))

            link = root / "artifacts" / "link.lean"
            link.symlink_to(root / "artifacts" / "Target.lean")
            lock = lock_for(root, artifact_path="artifacts/link.lean")
            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                formal_target.validate_target(root, formal_target.FormalTargetLock.from_mapping(lock))

    def test_wrong_sha256_or_byte_count_fails(self) -> None:
        directory, root = self.make_root()
        with directory:
            lock = self.write_valid_target(root)
            bad_hash = dict(lock)
            bad_hash["artifacts"] = [dict(lock["artifacts"][0], sha256="c" * 64)]  # type: ignore[index]
            with self.assertRaisesRegex(protocol.ValidationError, "sha256"):
                formal_target.validate_target(root, formal_target.FormalTargetLock.from_mapping(bad_hash))

            bad_bytes = dict(lock)
            bad_bytes["artifacts"] = [dict(lock["artifacts"][0], bytes=999)]  # type: ignore[index]
            with self.assertRaisesRegex(protocol.ValidationError, "byte"):
                formal_target.validate_target(root, formal_target.FormalTargetLock.from_mapping(bad_bytes))

    def test_symlinked_root_or_ancestor_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory).resolve()
            real = base / "real"
            real.mkdir()
            (real / "artifacts").mkdir()
            (real / "ledger").mkdir()
            (real / "artifacts" / "Target.lean").write_bytes(b"target\n")
            formal_target.append_ledger_row(real / "ledger", formal_target.LedgerRow.from_mapping(row()))
            lock = formal_target.FormalTargetLock.from_mapping(lock_for(real))

            root_link = base / "root-link"
            root_link.symlink_to(real)
            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                formal_target.validate_target(root_link, lock)

            ancestor = base / "ancestor"
            ancestor.mkdir()
            nested_real = ancestor / "nested-real"
            nested_real.mkdir()
            link_parent = base / "link-parent"
            link_parent.symlink_to(ancestor)
            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                formal_target.validate_target(link_parent / "nested-real", lock)

    def test_invalid_ledger_status_fails(self) -> None:
        with self.assertRaisesRegex(protocol.ValidationError, "status"):
            formal_target.LedgerRow.from_mapping(row(status="unknown"))

    def test_target_rejects_blocked_or_conjectural_dependencies(self) -> None:
        for status in ("blocked", "conjectural"):
            directory, root = self.make_root()
            with directory, self.subTest(status=status):
                (root / "artifacts" / "Target.lean").write_bytes(b"target\n")
                formal_target.append_ledger_row(
                    root / "ledger",
                    formal_target.LedgerRow.from_mapping(row(status=status)),
                )
                lock = formal_target.FormalTargetLock.from_mapping(lock_for(root))
                with self.assertRaisesRegex(protocol.ValidationError, "dependency"):
                    formal_target.validate_target(root, lock)

    def test_target_accepts_mathlib_available_and_locally_proved_dependencies(self) -> None:
        directory, root = self.make_root()
        with directory:
            (root / "artifacts" / "Target.lean").write_bytes(b"target\n")
            formal_target.append_ledger_row(
                root / "ledger",
                formal_target.LedgerRow.from_mapping(row(row_id="mathlib-row", status="mathlib_available")),
            )
            formal_target.append_ledger_row(
                root / "ledger",
                formal_target.LedgerRow.from_mapping(row(row_id="local-row", status="locally_proved")),
            )
            lock = lock_for(root)
            lock["target"] = dict(lock["target"], dependency_ids=["mathlib-row", "local-row"])  # type: ignore[index]

            target = formal_target.validate_target(root, formal_target.FormalTargetLock.from_mapping(lock))

            self.assertEqual(target.target_id, "crouzeix-main")
            self.assertEqual(sorted(target.ledger_rows), ["local-row", "mathlib-row"])

    def test_reappending_or_rewriting_existing_ledger_artifact_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory).resolve() / "ledger"
            ledger_row = formal_target.LedgerRow.from_mapping(row())
            first = formal_target.append_ledger_row(
                ledger,
                ledger_row,
            )

            row_sha256 = digest(
                json.dumps(
                    ledger_row.to_json(),
                    sort_keys=True,
                    separators=(",", ":"),
                    ensure_ascii=False,
                    allow_nan=False,
                ).encode("utf-8")
            )
            self.assertEqual(first.name, f"lemma-foundation.{row_sha256}.json")
            with self.assertRaisesRegex(protocol.ValidationError, "already exists"):
                formal_target.append_ledger_row(
                    ledger,
                    formal_target.LedgerRow.from_mapping(row()),
                )
            with self.assertRaisesRegex(protocol.ValidationError, "already exists"):
                formal_target.append_ledger_row(
                    ledger,
                    formal_target.LedgerRow.from_mapping(row(statement_sha256="d" * 64)),
                )
            self.assertEqual(first.read_text(encoding="utf-8").count("lemma-foundation"), 1)


if __name__ == "__main__":
    unittest.main()
