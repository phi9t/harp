from __future__ import annotations

import importlib.util
import json
import stat
import tempfile
import unittest
from pathlib import Path


def load_module(name: str, relative_path: str):
    module_path = Path(__file__).resolve().parents[2] / relative_path
    specification = importlib.util.spec_from_file_location(name, module_path)
    assert specification is not None
    assert specification.loader is not None
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


assets = load_module("validate_obsidian_assets", "scripts/validate_obsidian_assets.py")
profile_installer = load_module("apply_profile", "tools/obsidian/apply_profile.py")


class ObsidianAssetValidationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        self.write_valid_assets()

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def write_valid_assets(self) -> None:
        profile = self.root / "tools" / "obsidian" / "profile"
        (profile / "snippets").mkdir(parents=True)
        (self.root / "knowledge" / "rsi").mkdir(parents=True)
        (self.root / "knowledge" / "darwinx").mkdir(parents=True)
        (self.root / "knowledge" / "rsi" / "rsi_index.md").write_text(
            "# RSI\n", encoding="utf-8"
        )
        (self.root / "knowledge" / "darwinx" / "darwinx_index.md").write_text(
            "# DarwinX\n", encoding="utf-8"
        )
        (profile / "app.json").write_text("{}\n", encoding="utf-8")
        (profile / "core-plugins.json").write_text(
            json.dumps(sorted(assets.REQUIRED_CORE_PLUGINS)) + "\n",
            encoding="utf-8",
        )
        (profile / "bookmarks.json").write_text("[]\n", encoding="utf-8")
        (profile / "workspace.json").write_text("{}\n", encoding="utf-8")
        (profile / "snippets" / "harp-knowledge.css").write_text(
            ".obsidian-callout {}\n", encoding="utf-8"
        )
        navigation = self.root / "knowledge" / "obsidian"
        navigation.mkdir()
        (navigation / "harp_knowledge.base").write_text(
            """
filters: 'file.inFolder("knowledge") && file.ext == "md"'
properties:
  file.link:
    displayName: "Document"
views:
  - type: cards
    name: Reader routes
    order: [file.link]
""".lstrip(),
            encoding="utf-8",
        )
        (navigation / "harp_knowledge_map.canvas").write_text(
            json.dumps(
                {
                    "nodes": [
                        {
                            "id": "0123456789abcdef",
                            "type": "file",
                            "x": 0,
                            "y": 0,
                            "width": 300,
                            "height": 200,
                            "file": "knowledge/rsi/rsi_index.md",
                        },
                        {
                            "id": "fedcba9876543210",
                            "type": "file",
                            "x": 400,
                            "y": 0,
                            "width": 300,
                            "height": 200,
                            "file": "knowledge/darwinx/darwinx_index.md",
                        },
                    ],
                    "edges": [
                        {
                            "id": "0011223344556677",
                            "fromNode": "0123456789abcdef",
                            "toNode": "fedcba9876543210",
                            "label": "read next",
                        }
                    ],
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )

    def test_valid_assets_pass(self) -> None:
        assets.validate_repository_assets(self.root)

    def test_missing_required_profile_file_is_rejected(self) -> None:
        (self.root / "tools" / "obsidian" / "profile" / "app.json").unlink()

        with self.assertRaisesRegex(assets.ValidationError, "app.json"):
            assets.validate_repository_assets(self.root)

    def test_unapproved_profile_file_is_rejected(self) -> None:
        profile = self.root / "tools" / "obsidian" / "profile"
        (profile / "community-plugins.json").write_text("[]\n", encoding="utf-8")

        with self.assertRaisesRegex(assets.ValidationError, "not allowlisted"):
            assets.validate_repository_assets(self.root)

    def test_missing_required_core_plugin_is_rejected(self) -> None:
        plugins = sorted(assets.REQUIRED_CORE_PLUGINS - {"canvas"})
        (self.root / "tools" / "obsidian" / "profile" / "core-plugins.json").write_text(
            json.dumps(plugins) + "\n", encoding="utf-8"
        )

        with self.assertRaisesRegex(assets.ValidationError, "canvas"):
            assets.validate_repository_assets(self.root)

    def test_base_rejects_unknown_top_level_key_and_unsupported_view(self) -> None:
        base = self.root / "knowledge" / "obsidian" / "harp_knowledge.base"
        base.write_text(
            """
filters: 'file.inFolder("knowledge")'
unsupported: true
views:
  - type: timeline
    name: Invalid
""".lstrip(),
            encoding="utf-8",
        )

        with self.assertRaisesRegex(assets.ValidationError, "unsupported"):
            assets.validate_repository_assets(self.root)

    def test_canvas_rejects_duplicate_ids_dangling_edges_and_missing_files(self) -> None:
        canvas = self.root / "knowledge" / "obsidian" / "harp_knowledge_map.canvas"
        canvas.write_text(
            json.dumps(
                {
                    "nodes": [
                        {
                            "id": "0123456789abcdef",
                            "type": "file",
                            "x": 0,
                            "y": 0,
                            "width": 300,
                            "height": 200,
                            "file": "knowledge/missing.md",
                        },
                        {
                            "id": "0123456789abcdef",
                            "type": "text",
                            "x": 0,
                            "y": 0,
                            "width": 300,
                            "height": 200,
                            "text": "Duplicate",
                        },
                    ],
                    "edges": [
                        {
                            "id": "0011223344556677",
                            "fromNode": "0123456789abcdef",
                            "toNode": "aaaaaaaaaaaaaaaa",
                            "label": "read next",
                        }
                    ],
                }
            ),
            encoding="utf-8",
        )

        with self.assertRaisesRegex(assets.ValidationError, "duplicate"):
            assets.validate_repository_assets(self.root)


class ObsidianProfileInstallerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        self.profile = self.root / "profile"
        (self.profile / "snippets").mkdir(parents=True)
        (self.root / "vault").mkdir()
        self.vault = self.root / "vault"
        self.write_profile()

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def write_profile(self) -> None:
        (self.profile / "app.json").write_text("{}\n", encoding="utf-8")
        (self.profile / "core-plugins.json").write_text(
            json.dumps(sorted(assets.REQUIRED_CORE_PLUGINS)) + "\n",
            encoding="utf-8",
        )
        (self.profile / "bookmarks.json").write_text("[]\n", encoding="utf-8")
        (self.profile / "workspace.json").write_text("{}\n", encoding="utf-8")
        (self.profile / "snippets" / "harp-knowledge.css").write_text(
            ".canonical-table-scroll {}\n", encoding="utf-8"
        )

    def test_create_only_install_writes_allowlisted_files_and_receipt(self) -> None:
        receipt = profile_installer.apply_profile(self.vault, self.profile)
        destination = self.vault / ".obsidian"

        self.assertEqual(
            {
                path.relative_to(destination).as_posix()
                for path in destination.rglob("*")
                if path.is_file()
            },
            assets.REQUIRED_PROFILE_FILES | {"harp-profile-receipt.json"},
        )
        self.assertEqual(stat.S_IMODE(destination.stat().st_mode), 0o700)
        self.assertEqual(receipt["profile_version"], profile_installer.PROFILE_VERSION)
        self.assertEqual(set(receipt["files"]), assets.REQUIRED_PROFILE_FILES)
        self.assertIn("applied_at_utc", receipt)

        with self.assertRaisesRegex(profile_installer.ProfileError, "already exists"):
            profile_installer.apply_profile(self.vault, self.profile)

    def test_replace_updates_allowlisted_file_without_touching_personal_state(self) -> None:
        destination = self.vault / ".obsidian"
        destination.mkdir()
        (destination / "app.json").write_text('{"old": true}\n', encoding="utf-8")
        (destination / "personal.json").write_text('{"keep": true}\n', encoding="utf-8")

        profile_installer.apply_profile(self.vault, self.profile, replace=True)

        self.assertEqual((destination / "app.json").read_text(encoding="utf-8"), "{}\n")
        self.assertEqual(
            (destination / "personal.json").read_text(encoding="utf-8"),
            '{"keep": true}\n',
        )

    def test_rejects_symlinked_source_vault_and_destination_component(self) -> None:
        source_target = self.root / "source-target.json"
        source_target.write_text("{}\n", encoding="utf-8")
        (self.profile / "app.json").unlink()
        (self.profile / "app.json").symlink_to(source_target)
        with self.assertRaisesRegex(profile_installer.ProfileError, "symlink"):
            profile_installer.apply_profile(self.vault, self.profile)

        self.write_profile()
        vault_link = self.root / "vault-link"
        vault_link.symlink_to(self.vault, target_is_directory=True)
        with self.assertRaisesRegex(profile_installer.ProfileError, "symlink"):
            profile_installer.apply_profile(vault_link, self.profile)

        destination = self.vault / ".obsidian"
        destination.mkdir()
        (destination / "snippets").symlink_to(self.root, target_is_directory=True)
        with self.assertRaisesRegex(profile_installer.ProfileError, "symlink"):
            profile_installer.apply_profile(self.vault, self.profile, replace=True)
