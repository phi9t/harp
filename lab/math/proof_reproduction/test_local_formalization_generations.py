from __future__ import annotations

import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import test_local_formalization_evidence as fixtures
import local_formalization_validation as validation
from test_local_formalization_evidence import (
    FakeExecutor, make_workspace,
    local_formalization_evidence as evidence,
)


class LocalGenerationTests(unittest.TestCase):
    def test_first_refresh_failure_restores_legacy_only_state(self):
        for failure in ("executor", "preflight"):
            with self.subTest(failure=failure):
                root, base = self.fixture()
                legacy = base / evidence.DESTINATION_NAME / evidence.MANIFEST_NAME
                before = legacy.read_bytes()
                def fail(*args):
                    raise RuntimeError("first refresh failed")
                boundary = mock.patch.object(evidence, "_snapshot_build_state", side_effect=fail) if failure == "preflight" else mock.patch.object(evidence, "_shared_xdg_environment", return_value={})
                with boundary:
                    with self.assertRaisesRegex(RuntimeError, "first refresh failed"):
                        self.refresh(root, fail)
                self.assertFalse((base / evidence.SELECTION_NAME).exists())
                self.assertFalse((base / evidence.GENERATIONS_NAME).exists())
                self.assertEqual(legacy.read_bytes(), before)
                self.assertEqual(validation.validate_local_formalization_bundle(root).manifest_path, legacy)

    def test_failed_refresh_preserves_preexisting_empty_generation_root(self):
        root, base = self.fixture()
        generations = base / evidence.GENERATIONS_NAME
        generations.mkdir()
        identity = generations.stat().st_ino
        def fail(*args):
            raise RuntimeError("first refresh failed")
        with self.assertRaisesRegex(RuntimeError, "first refresh failed"):
            self.refresh(root, fail)
        self.assertEqual(generations.stat().st_ino, identity)

    def test_failed_first_refresh_never_removes_visible_generation_entries(self):
        root, base = self.fixture()
        visible = base / evidence.GENERATIONS_NAME / ("e" * 32)
        def fail(*args):
            visible.mkdir()
            (visible / "captured.txt").write_bytes(b"preserve")
            raise RuntimeError("first refresh failed")
        with self.assertRaisesRegex(RuntimeError, "first refresh failed"):
            self.refresh(root, fail)
        self.assertEqual((visible / "captured.txt").read_bytes(), b"preserve")

    def test_reader_returns_selected_manifest_and_rejects_later_tampering(self):
        root, legacy = self.fixture()
        publication = evidence.refresh_local_formalization_evidence(root, executor=FakeExecutor())
        result = validation.validate_local_formalization_bundle(root)
        self.assertEqual(result.manifest_path, publication.manifest_path)
        self.assertNotEqual(result.manifest_path, legacy / "local_formalization/manifest.tsv")
        publication.manifest_path.write_bytes(publication.manifest_path.read_bytes() + b"\n")
        with self.assertRaisesRegex(evidence.protocol.ValidationError, "digest mismatch"):
            validation.validate_local_formalization_bundle(root)

    setUp = fixtures.LocalFormalizationEvidenceTests.setUp

    def fixture(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root, receipt, _ = make_workspace(Path(directory.name).resolve())
        patcher = mock.patch.object(evidence.ls_validation, "validate_committed_receipts",
                                    return_value={"ls-terminal-crouzeix": receipt})
        patcher.start()
        self.addCleanup(patcher.stop)
        evidence.publish_local_formalization_evidence(root, executor=FakeExecutor())
        return root, root / "evidence/crouzeix_conjecture"

    def refresh(self, root, executor=None):
        return evidence.refresh_local_formalization_evidence(root, executor=executor or FakeExecutor())

    def test_two_refreshes_preserve_legacy_and_capture_new_execution(self):
        root, base = self.fixture()
        legacy = {str(p.relative_to(base)): p.read_bytes() for p in (base / "local_formalization").rglob("*") if p.is_file()}
        first_run, second_run = FakeExecutor(), FakeExecutor()
        first = self.refresh(root, first_run)
        second = self.refresh(root, second_run)
        self.assertNotEqual(first.artifact_root, second.artifact_root)
        self.assertEqual(len(first_run.calls), 7)
        self.assertEqual(len(second_run.calls), 7)
        pointer = json.loads((base / "local_formalization.current.json").read_text())
        self.assertEqual(pointer["active_generation"], second.artifact_root.name)
        self.assertEqual(len(pointer["generations"]), 3)
        for name, data in legacy.items():
            self.assertEqual((base / name).read_bytes(), data)
        self.assertIn(f"local_formalization_generations/{first.artifact_root.name}/build/", first.manifest_path.read_text())

    def test_invalid_pointer_never_falls_back_or_runs_executor(self):
        for raw in [b'{}', b'{"schema_version":1,"schema_version":2}', b'not-json']:
            with self.subTest(raw=raw):
                root, base = self.fixture()
                (base / "local_formalization.current.json").write_bytes(raw)
                run = FakeExecutor()
                with self.assertRaises(evidence.protocol.ValidationError):
                    self.refresh(root, run)
                self.assertEqual(run.calls, [])

    def test_tampered_historical_bundle_rejects_refresh(self):
        root, base = self.fixture()
        self.refresh(root)
        (base / "local_formalization/build/stdout.log").write_text("tampered")
        run = FakeExecutor()
        with self.assertRaisesRegex(evidence.protocol.ValidationError, "digest"):
            self.refresh(root, run)
        self.assertEqual(run.calls, [])

    def test_unregistered_generation_rejects_refresh(self):
        root, base = self.fixture()
        rogue = base / "local_formalization_generations" / ("a" * 32)
        rogue.mkdir(parents=True)
        with self.assertRaisesRegex(evidence.protocol.ValidationError, "generation"):
            self.refresh(root)

    def test_selection_failure_reports_preserved_visible_generation(self):
        root, base = self.fixture()
        first = self.refresh(root)
        before = (base / "local_formalization.current.json").read_bytes()
        with mock.patch.object(evidence, "_select_generation", side_effect=OSError("selection failed")):
            with self.assertRaises(evidence.PublicationCommittedError) as caught:
                self.refresh(root)
        self.assertTrue(caught.exception.recovery_path.is_dir())
        self.assertNotEqual(caught.exception.recovery_path, first.artifact_root)
        self.assertEqual((base / "local_formalization.current.json").read_bytes(), before)

    def test_generation_pointer_symlink_rejected(self):
        root, base = self.fixture()
        target = root / "external-pointer"
        target.write_text("{}")
        (base / "local_formalization.current.json").symlink_to(target)
        with self.assertRaises(evidence.protocol.ValidationError):
            self.refresh(root)

    def test_executor_failure_preserves_selection_and_all_bundles(self):
        root, base = self.fixture()
        self.refresh(root)
        before = {str(p.relative_to(base)): p.read_bytes() for p in base.rglob("*") if p.is_file()}
        def fail(*args):
            raise RuntimeError("fake execution failed")
        with self.assertRaisesRegex(RuntimeError, "fake execution failed"):
            self.refresh(root, fail)
        after = {str(p.relative_to(base)): p.read_bytes() for p in base.rglob("*") if p.is_file()}
        self.assertEqual(after, before)

    def test_refresh_uses_existing_publication_lock(self):
        root, base = self.fixture()
        lock = evidence.ls_receipts._acquire_publication_lock(base)
        try:
            run = FakeExecutor()
            with self.assertRaises(evidence.protocol.ValidationError):
                self.refresh(root, run)
            self.assertEqual(run.calls, [])
        finally:
            evidence.ls_receipts._release_publication_lock(lock)

    def test_generation_roster_changed_during_execution_is_rejected(self):
        root, base = self.fixture()
        self.refresh(root)
        before = (base / evidence.SELECTION_NAME).read_bytes()
        run = FakeExecutor()
        def inject(*args):
            result = run(*args)
            if len(run.calls) == 1:
                (base / evidence.GENERATIONS_NAME / ("f" * 32)).mkdir()
            return result
        with self.assertRaisesRegex(evidence.protocol.ValidationError, "generation"):
            self.refresh(root, inject)
        self.assertEqual((base / evidence.SELECTION_NAME).read_bytes(), before)

    def test_generation_roster_changed_during_selection_read_is_rejected(self):
        root, base = self.fixture()
        self.refresh(root)
        original = evidence._bundle_member_snapshots
        injected = False
        def inject(*args):
            nonlocal injected
            result = original(*args)
            if not injected:
                injected = True
                (base / evidence.GENERATIONS_NAME / ("f" * 32)).mkdir()
            return result
        with mock.patch.object(evidence, "_bundle_member_snapshots", side_effect=inject):
            with self.assertRaisesRegex(evidence.protocol.ValidationError, "roster"):
                validation.validate_local_formalization_bundle(root)


if __name__ == "__main__":
    unittest.main()
