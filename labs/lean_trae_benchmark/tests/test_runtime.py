from __future__ import annotations

import dataclasses
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


sys.path.insert(0, str(Path(__file__).parents[1]))

from model import Digest, RuntimeLock, ValidationError, sha256_bytes
import runtime


class RuntimeTests(unittest.TestCase):
    def _artifact(
        self,
        root: Path,
        *,
        version_stdout: str = "traecli 0.200.19(internal edition)",
    ) -> Path:
        artifact = root / "supplied-traecli"
        artifact.write_text(f"#!/bin/sh\nprintf '%s\\n' '{version_stdout}'\n", encoding="utf-8")
        artifact.chmod(0o755)
        return artifact

    def _lock_for(
        self,
        artifact: Path,
        *,
        version_stdout: str = "traecli 0.200.19(internal edition)",
    ) -> RuntimeLock:
        data = artifact.read_bytes()
        return RuntimeLock.from_value(
            {
                "schema_version": "harp-lean-runtime-lock/v1",
                "runtime_id": "lean-trae-test-v1",
                "platform": {"os": "darwin", "arch": "arm64"},
                "trae": {
                    "version": "0.200.19",
                    "artifact": {
                        "source_url": "https://example.test/traecli-0.200.19-darwin-arm64",
                        "digest": sha256_bytes(data).to_value(),
                        "bytes": len(data),
                    },
                    "published_path": "runtime/darwin-arm64/traecli",
                    "version_stdout": version_stdout,
                },
                "lean": {
                    "version": "4.19.0",
                    "toolchain_id": "leanprover/lean4:v4.19.0",
                    "artifact_digest": Digest("sha256", "1" * 64).to_value(),
                    "elan_layout_digest": Digest("sha256", "2" * 64).to_value(),
                },
                "mathlib": None,
            },
            "test runtime lock",
        )

    def _darwin_arm64(self):
        return mock.patch.multiple(
            runtime.platform,
            system=mock.Mock(return_value="Darwin"),
            machine=mock.Mock(return_value="arm64"),
        )

    def _base(self, parent: Path) -> Path:
        base = parent / "harp-runtime-base"
        base.mkdir()
        return base

    def _published_root(self, lock: RuntimeLock, base: Path) -> Path:
        return base / Path(lock.trae.published_path).parent

    def test_provision_copies_only_locked_regular_artifact_and_resolves_it(self) -> None:
        with tempfile.TemporaryDirectory() as directory, self._darwin_arm64():
            parent = Path(directory)
            artifact = self._artifact(parent)
            lock = self._lock_for(artifact)
            base = self._base(parent)
            root = self._published_root(lock, base)

            provisioned = runtime.provision_runtime(lock, artifact, base)
            resolved = runtime.resolve_runtime(lock, base)

            self.assertEqual(provisioned.trae, root / "traecli")
            self.assertEqual(resolved, provisioned)
            self.assertEqual((root / "traecli").read_bytes(), artifact.read_bytes())
            self.assertFalse((root / "traecli").is_symlink())
            inventory = json.loads((root / "inventory.json").read_text(encoding="utf-8"))
            self.assertEqual(
                inventory["files"],
                [
                    {
                        "bytes": artifact.stat().st_size,
                        "path": "traecli",
                        "sha256": sha256_bytes(artifact.read_bytes()).hex,
                    }
                ],
            )

    def test_provision_rejects_wrong_sha_and_wrong_byte_count(self) -> None:
        with tempfile.TemporaryDirectory() as directory, self._darwin_arm64():
            parent = Path(directory)
            artifact = self._artifact(parent)
            lock = self._lock_for(artifact)
            base = self._base(parent)
            wrong_digest = dataclasses.replace(
                lock,
                trae=dataclasses.replace(
                    lock.trae,
                    artifact=dataclasses.replace(lock.trae.artifact, digest=Digest("sha256", "0" * 64)),
                ),
            )
            wrong_bytes = dataclasses.replace(
                lock,
                trae=dataclasses.replace(
                    lock.trae,
                    artifact=dataclasses.replace(
                        lock.trae.artifact,
                        bytes=artifact.stat().st_size + 1,
                    ),
                ),
            )

            for index, broken in enumerate((wrong_digest, wrong_bytes)):
                with self.subTest(index=index), self.assertRaisesRegex(ValidationError, "artifact"):
                    runtime.provision_runtime(broken, artifact, base)

    def test_provision_rejects_symlinks_stale_staging_and_destination_overwrite(self) -> None:
        with tempfile.TemporaryDirectory() as directory, self._darwin_arm64():
            parent = Path(directory)
            artifact = self._artifact(parent)
            lock = self._lock_for(artifact)
            base = self._base(parent)
            root = self._published_root(lock, base)

            link = parent / "artifact-link"
            link.symlink_to(artifact)
            with self.assertRaisesRegex(ValidationError, "artifact.*symlink"):
                runtime.provision_runtime(lock, link, base)

            root.parent.mkdir()
            (root.parent / f".{root.name}.runtime-stage-interrupted").mkdir()
            with self.assertRaisesRegex(ValidationError, "staging"):
                runtime.provision_runtime(lock, artifact, base)

            (root.parent / f".{root.name}.runtime-stage-interrupted").rmdir()
            root.mkdir()
            with self.assertRaisesRegex(ValidationError, "destination"):
                runtime.provision_runtime(lock, artifact, base)

    def test_provision_rejects_wrong_version_unsupported_platform_and_symlinked_destination(self) -> None:
        with tempfile.TemporaryDirectory() as directory, self._darwin_arm64():
            parent = Path(directory)
            base = self._base(parent)
            wrong_version_artifact = self._artifact(parent, version_stdout="traecli 0.200.20")
            lock = self._lock_for(wrong_version_artifact)
            with self.assertRaisesRegex(ValidationError, "version stdout"):
                runtime.provision_runtime(lock, wrong_version_artifact, base)

            artifact = self._artifact(parent, version_stdout="traecli 0.200.19(internal edition)")
            good_lock = self._lock_for(artifact)
            with mock.patch.object(runtime.platform, "system", return_value="Linux"):
                with self.assertRaisesRegex(ValidationError, "platform"):
                    runtime.provision_runtime(good_lock, artifact, base)

            real_destination = parent / "real-destination"
            real_destination.mkdir()
            expected_root = self._published_root(good_lock, base)
            expected_root.parent.mkdir(exist_ok=True)
            destination_link = expected_root
            destination_link.symlink_to(real_destination, target_is_directory=True)
            with self.assertRaisesRegex(ValidationError, "destination.*symlink"):
                runtime.provision_runtime(good_lock, artifact, base)

            real_parent = parent / "real-parent"
            real_parent.mkdir()
            parent_link = parent / "parent-link"
            parent_link.symlink_to(real_parent, target_is_directory=True)
            with self.assertRaisesRegex(ValidationError, "base.*symlink"):
                runtime.provision_runtime(good_lock, artifact, parent_link)

    def test_post_publication_validation_failure_rolls_back_new_destination(self) -> None:
        with tempfile.TemporaryDirectory() as directory, self._darwin_arm64():
            parent = Path(directory)
            artifact = self._artifact(parent)
            lock = self._lock_for(artifact)
            base = self._base(parent)
            root = self._published_root(lock, base)
            original_verify = runtime._verify_runtime_tree

            def fail_after_publish(path: Path, checked_lock: RuntimeLock) -> None:
                if path == root:
                    raise ValidationError("injected post-publication failure")
                original_verify(path, checked_lock)

            with mock.patch.object(runtime, "_verify_runtime_tree", side_effect=fail_after_publish):
                with self.assertRaisesRegex(ValidationError, "injected post-publication"):
                    runtime.provision_runtime(lock, artifact, base)

            self.assertFalse(root.exists())
            self.assertEqual(list(root.parent.glob(f".{root.name}.runtime-stage-*")), [])

    def test_competing_destination_created_at_publish_survives(self) -> None:
        with tempfile.TemporaryDirectory() as directory, self._darwin_arm64():
            parent = Path(directory)
            artifact = self._artifact(parent)
            lock = self._lock_for(artifact)
            base = self._base(parent)
            root = self._published_root(lock, base)
            original_publish = runtime._rename_directory_no_replace

            def create_competitor_then_publish(stage: Path, destination: Path) -> None:
                destination.mkdir()
                original_publish(stage, destination)

            with mock.patch.object(
                runtime,
                "_rename_directory_no_replace",
                side_effect=create_competitor_then_publish,
            ):
                with self.assertRaisesRegex(ValidationError, "destination"):
                    runtime.provision_runtime(lock, artifact, base)

            self.assertTrue(root.is_dir())
            self.assertEqual(list(root.iterdir()), [])
            self.assertEqual(list(root.parent.glob(f".{root.name}.runtime-stage-*")), [])

    def test_no_replace_publication_has_no_unsafe_non_darwin_fallback(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            stage = parent / "stage"
            destination = parent / "destination"
            stage.mkdir()
            with mock.patch.object(runtime.platform, "system", return_value="Linux"):
                with self.assertRaisesRegex(ValidationError, "no-replace.*unsupported"):
                    runtime._rename_directory_no_replace(stage, destination)
            self.assertTrue(stage.is_dir())
            self.assertFalse(destination.exists())

    def test_resolver_accepts_only_the_lock_derived_location_below_a_runtime_base(self) -> None:
        with tempfile.TemporaryDirectory() as directory, self._darwin_arm64():
            parent = Path(directory)
            artifact = self._artifact(parent)
            lock = self._lock_for(artifact)
            base = self._base(parent)
            resolved = runtime.provision_runtime(lock, artifact, base)

            alternate_root = base / "alternate-runtime"
            alternate_root.mkdir()
            for source in (resolved.trae, resolved.inventory):
                (alternate_root / source.name).write_bytes(source.read_bytes())
            (alternate_root / "traecli").chmod(0o755)

            self.assertEqual(runtime.resolve_runtime(lock, base).root, resolved.root)
            with self.assertRaisesRegex(ValidationError, "runtime directory"):
                runtime.resolve_runtime(lock, alternate_root)

    def test_source_mutation_during_copy_is_rejected_before_staged_binary_execution(self) -> None:
        with tempfile.TemporaryDirectory() as directory, self._darwin_arm64():
            parent = Path(directory)
            marker = parent / "attacker-executed"
            artifact = self._artifact(parent)
            lock = self._lock_for(artifact)
            base = self._base(parent)
            original_copy = runtime._copy_regular_file

            def replace_source_then_copy(source: Path, target: Path) -> None:
                source.write_text(
                    "#!/bin/sh\n"
                    f"touch '{marker}'\n"
                    "printf '%s\\n' 'traecli 0.200.19(internal edition)'\n",
                    encoding="utf-8",
                )
                source.chmod(0o755)
                original_copy(source, target)

            with mock.patch.object(runtime, "_copy_regular_file", side_effect=replace_source_then_copy):
                with self.assertRaisesRegex(ValidationError, "byte size|digest"):
                    runtime.provision_runtime(lock, artifact, base)

            self.assertFalse(marker.exists())

    def test_load_production_lock_is_not_redirected_by_ambient_environment(self) -> None:
        expected = runtime.load_runtime_lock(runtime.DEFAULT_RUNTIME_LOCK)
        with tempfile.TemporaryDirectory() as directory:
            redirect = Path(directory) / "redirect.json"
            redirect.write_text("{}", encoding="utf-8")
            with mock.patch.dict(os.environ, {"HARP_LEAN_TRAE_RUNTIME_LOCK": str(redirect)}):
                self.assertEqual(runtime.load_runtime_lock(runtime.DEFAULT_RUNTIME_LOCK), expected)

    def test_committed_lock_is_darwin_arm64_and_has_the_real_trae_pin(self) -> None:
        lock = runtime.load_runtime_lock(runtime.DEFAULT_RUNTIME_LOCK)
        self.assertEqual((lock.os, lock.arch), ("darwin", "arm64"))
        self.assertEqual(lock.trae.version, "0.200.19")
        self.assertEqual(lock.trae.artifact.digest.hex, "2de3b4a458c219953a9eb044a4e07dcae18f150637d5d83874f38a4ae57bed57")
        self.assertEqual(lock.trae.artifact.bytes, 239733536)
        self.assertEqual(lock.lean.version, "4.19.0")
        self.assertIsNone(lock.mathlib)


if __name__ == "__main__":
    unittest.main()
