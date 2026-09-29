import io
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from sync_agentic_workflow import END_MARKER, START_MARKER, run


class SyncAgenticWorkflowTest(unittest.TestCase):
    def test_repository_manifest_manages_existing_skill_orchestration(self):
        repository_root = Path(__file__).resolve().parents[1]
        manifest = json.loads(
            (repository_root / "docs/agents/agentic-workflow-repos.json").read_text()
        )
        relative_path = "docs/agents/skill-orchestration.md"

        self.assertIn(relative_path, manifest["managed_policy_files"])
        self.assertTrue((repository_root / relative_path).is_file())
        repository_names = {
            repository["name"] for repository in manifest["repositories"]
        }
        self.assertNotIn("tnsr", repository_names)
        self.assertIn("vaso", repository_names)
        self.assertIn("miles_diffusion", repository_names)

    def test_repository_manifest_manages_cutlass(self):
        repository_root = Path(__file__).resolve().parents[1]
        manifest = json.loads(
            (repository_root / "docs/agents/agentic-workflow-repos.json").read_text()
        )
        repositories = {
            repository["name"]: repository
            for repository in manifest["repositories"]
        }

        self.assertEqual(
            repositories["cutlass"],
            {
                "name": "cutlass",
                "instruction_files": ["AGENTS.md"],
                "install_local_tracker": True,
                "base_branch": "main",
            },
        )

    def setUp(self):
        self.tempdir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tempdir.cleanup)
        self.workspace = Path(self.tempdir.name)
        self.source = self.workspace / "ultron"
        self.peer = self.workspace / "peer"
        self.source.mkdir()
        self.peer.mkdir()

        self.policy_files = {
            "CONSTITUTION.md": "constitution\n",
            "docs/agents/agentic-engineering.md": "workflow\n",
            "docs/agents/skill-orchestration.md": "crosswalk\n",
        }
        self.tracker_files = {
            "docs/agents/issue-tracker.md": "tracker\n",
            "docs/agents/triage-labels.md": "labels\n",
            "docs/agents/domain.md": "domain\n",
        }
        for relative_path, content in {
            **self.policy_files,
            **self.tracker_files,
        }.items():
            path = self.source / relative_path
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)

        (self.source / "AGENTS.md").write_text("source guidance\n")
        (self.peer / "AGENTS.md").write_text("peer guidance\n")
        self.manifest_path = self.source / "docs/agents/agentic-workflow-repos.json"
        self.manifest_path.write_text(
            json.dumps(
                {
                    "version": "2026.08.14.1",
                    "source_repository": "ultron",
                    "managed_policy_files": list(self.policy_files),
                    "local_tracker_files": list(self.tracker_files),
                    "repositories": [
                        {
                            "name": "ultron",
                            "instruction_files": ["AGENTS.md"],
                            "install_local_tracker": False,
                        },
                        {
                            "name": "peer",
                            "instruction_files": ["AGENTS.md"],
                            "install_local_tracker": True,
                        },
                    ],
                }
            )
        )

    def invoke(self, mode):
        output = io.StringIO()
        status = run(
            mode=mode,
            workspace_root=self.workspace,
            manifest_path=self.manifest_path,
            output=output,
        )
        return status, output.getvalue()

    def test_apply_installs_managed_files_and_preserves_surrounding_guidance(self):
        status, output = self.invoke("apply")

        self.assertEqual(status, 0, output)
        for relative_path, content in {
            **self.policy_files,
            **self.tracker_files,
        }.items():
            self.assertEqual((self.peer / relative_path).read_text(), content)
        peer_instructions = (self.peer / "AGENTS.md").read_text()
        self.assertTrue(peer_instructions.startswith("peer guidance\n"))
        self.assertEqual(peer_instructions.count(START_MARKER), 1)
        self.assertEqual(peer_instructions.count(END_MARKER), 1)

        first_snapshot = {
            path.relative_to(self.workspace): path.read_bytes()
            for path in self.workspace.rglob("*")
            if path.is_file()
        }
        status, output = self.invoke("apply")
        second_snapshot = {
            path.relative_to(self.workspace): path.read_bytes()
            for path in self.workspace.rglob("*")
            if path.is_file()
        }
        self.assertEqual(status, 0, output)
        self.assertEqual(second_snapshot, first_snapshot)

    def test_apply_creates_missing_instruction_files_world_readable(self):
        new_peer = self.workspace / "new-peer"
        new_peer.mkdir()
        manifest = json.loads(self.manifest_path.read_text())
        manifest["repositories"].append(
            {
                "name": "new-peer",
                "instruction_files": ["AGENTS.md"],
                "install_local_tracker": False,
            }
        )
        self.manifest_path.write_text(json.dumps(manifest))

        status, output = self.invoke("apply")

        self.assertEqual(status, 0, output)
        instructions = new_peer / "AGENTS.md"
        self.assertTrue(instructions.is_file())
        self.assertEqual(instructions.stat().st_mode & 0o777, 0o644)

    def test_check_reports_drift_and_apply_repairs_it(self):
        self.assertEqual(self.invoke("apply")[0], 0)
        workflow = self.peer / "docs/agents/agentic-engineering.md"
        workflow.write_text("locally edited\n")

        status, output = self.invoke("check")
        self.assertEqual(status, 1)
        self.assertIn("DRIFT peer/docs/agents/agentic-engineering.md", output)

        status, output = self.invoke("apply")
        self.assertEqual(status, 0, output)
        self.assertEqual(workflow.read_text(), "workflow\n")
        self.assertEqual(self.invoke("check")[0], 0)

        source_workflow = self.source / "docs/agents/agentic-engineering.md"
        source_workflow.chmod(0o600)
        workflow.chmod(0o600)

        status, output = self.invoke("check")
        self.assertEqual(status, 1)
        self.assertIn("DRIFT peer/docs/agents/agentic-engineering.md", output)

        status, output = self.invoke("apply")
        self.assertEqual(status, 0, output)
        self.assertEqual(workflow.stat().st_mode & 0o777, 0o644)
        self.assertEqual(self.invoke("check")[0], 0)

    def test_refuses_repository_outside_workspace(self):
        manifest = json.loads(self.manifest_path.read_text())
        manifest["repositories"][1]["name"] = "../outside"
        self.manifest_path.write_text(json.dumps(manifest))

        status, output = self.invoke("apply")

        self.assertEqual(status, 2)
        self.assertIn("outside the workspace", output)

    def test_refuses_missing_repository(self):
        manifest = json.loads(self.manifest_path.read_text())
        manifest["repositories"][1]["name"] = "missing"
        self.manifest_path.write_text(json.dumps(manifest))

        status, output = self.invoke("check")

        self.assertEqual(status, 2)
        self.assertIn("repository is missing", output)

    def test_refuses_duplicate_or_unbalanced_instruction_markers(self):
        malformed = f"peer guidance\n{START_MARKER}\n{START_MARKER}\n{END_MARKER}\n"
        instructions = self.peer / "AGENTS.md"
        instructions.write_text(malformed)

        status, output = self.invoke("apply")

        self.assertEqual(status, 2)
        self.assertIn("malformed managed block", output)
        self.assertEqual(instructions.read_text(), malformed)

    def test_refuses_unexpected_instruction_symlink_topology(self):
        self.assertEqual(self.invoke("apply")[0], 0)
        manifest = json.loads(self.manifest_path.read_text())
        manifest["required_symlinks"] = [
            {"path": "peer/CLAUDE.md", "target": "AGENTS.md"}
        ]
        self.manifest_path.write_text(json.dumps(manifest))
        (self.peer / "CLAUDE.md").symlink_to("WRONG.md")

        status, output = self.invoke("check")

        self.assertEqual(status, 2)
        self.assertIn("unexpected symlink target", output)

    def test_refuses_missing_required_import(self):
        self.assertEqual(self.invoke("apply")[0], 0)
        manifest = json.loads(self.manifest_path.read_text())
        manifest["required_content"] = [
            {"path": "peer/AGENTS.md", "snippets": ["@CONSTITUTION.md"]}
        ]
        self.manifest_path.write_text(json.dumps(manifest))

        status, output = self.invoke("check")

        self.assertEqual(status, 2)
        self.assertIn("required content is missing", output)

    def test_refuses_dangling_symlink_at_managed_destination(self):
        destination = self.peer / "CONSTITUTION.md"
        destination.symlink_to("missing-constitution.md")

        status, output = self.invoke("apply")

        self.assertEqual(status, 2)
        self.assertIn("managed path contains a symlink", output)
        self.assertTrue(destination.is_symlink())

    def test_refuses_dangling_symlink_at_instruction_destination(self):
        destination = self.peer / "AGENTS.md"
        destination.unlink()
        destination.symlink_to("missing-agents.md")

        status, output = self.invoke("apply")

        self.assertEqual(status, 2)
        self.assertIn("instruction path contains a symlink", output)
        self.assertTrue(destination.is_symlink())

    def test_refuses_vcs_metadata_paths(self):
        source_config = self.source / ".git/config"
        peer_config = self.peer / ".git/config"
        source_config.parent.mkdir()
        peer_config.parent.mkdir()
        source_config.write_text("source config\n")
        peer_config.write_text("peer config\n")
        manifest = json.loads(self.manifest_path.read_text())
        manifest["managed_policy_files"] = [".git/config"]
        self.manifest_path.write_text(json.dumps(manifest))

        status, output = self.invoke("apply")

        self.assertEqual(status, 2)
        self.assertIn("VCS metadata", output)
        self.assertEqual(peer_config.read_text(), "peer config\n")

    def test_refuses_symlinked_repository_root(self):
        actual = self.workspace / "actual"
        actual.mkdir()
        (actual / "AGENTS.md").write_text("actual guidance\n")
        (self.workspace / "alias").symlink_to(actual, target_is_directory=True)
        manifest = json.loads(self.manifest_path.read_text())
        manifest["repositories"][1]["name"] = "alias"
        self.manifest_path.write_text(json.dumps(manifest))

        status, output = self.invoke("apply")

        self.assertEqual(status, 2)
        self.assertIn("repository root must not be a symlink", output)
        self.assertFalse((actual / "CONSTITUTION.md").exists())

    def test_refuses_symlinked_managed_path_ancestor(self):
        real_docs = self.peer / "real-docs"
        real_docs.mkdir()
        (self.peer / "docs").symlink_to(real_docs, target_is_directory=True)

        status, output = self.invoke("apply")

        self.assertEqual(status, 2)
        self.assertIn("managed path contains a symlink", output)
        self.assertFalse((real_docs / "agents/agentic-engineering.md").exists())


class MainlineBranchTest(unittest.TestCase):
    def _git(self, repository, *args):
        subprocess.run(
            ["git", *args],
            cwd=repository,
            check=True,
            capture_output=True,
            text=True,
        )

    def _init_repo(self, repository, base_branch):
        repository.mkdir(parents=True, exist_ok=True)
        self._git(repository, "init", "-q", "-b", base_branch)
        self._git(repository, "config", "user.email", "test@example.com")
        self._git(repository, "config", "user.name", "Test")
        (repository / "AGENTS.md").write_text("guidance\n")
        self._git(repository, "add", "AGENTS.md")
        self._git(repository, "commit", "-q", "-m", "initial")

    def _rev(self, repository, ref):
        result = subprocess.run(
            ["git", "rev-parse", ref],
            cwd=repository,
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()

    def _branch_exists(self, repository, branch):
        result = subprocess.run(
            ["git", "show-ref", "--verify", "--quiet", f"refs/heads/{branch}"],
            cwd=repository,
        )
        return result.returncode == 0

    def setUp(self):
        self.tempdir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tempdir.cleanup)
        self.workspace = Path(self.tempdir.name)
        self.source = self.workspace / "ultron"
        self.peer = self.workspace / "peer"
        self._init_repo(self.source, "main")
        self._init_repo(self.peer, "master")

        policy = {"CONSTITUTION.md": "constitution\n"}
        for relative_path, content in policy.items():
            path = self.source / relative_path
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)

        self.manifest_path = self.source / "docs/agents/agentic-workflow-repos.json"
        self.manifest = {
            "version": "2026.09.02.1",
            "source_repository": "ultron",
            "mainline_branch": "ultron/mainline",
            "managed_policy_files": ["CONSTITUTION.md"],
            "local_tracker_files": [],
            "repositories": [
                {
                    "name": "ultron",
                    "instruction_files": ["AGENTS.md"],
                    "install_local_tracker": False,
                    "base_branch": "main",
                },
                {
                    "name": "peer",
                    "instruction_files": ["AGENTS.md"],
                    "install_local_tracker": False,
                    "base_branch": "master",
                },
            ],
        }

    def _write_manifest(self):
        self.manifest_path.parent.mkdir(parents=True, exist_ok=True)
        self.manifest_path.write_text(json.dumps(self.manifest))

    def invoke(self, mode):
        output = io.StringIO()
        status = run(
            mode=mode,
            workspace_root=self.workspace,
            manifest_path=self.manifest_path,
            output=output,
        )
        return status, output.getvalue()

    def test_check_flags_missing_mainline_branch(self):
        self._write_manifest()

        status, output = self.invoke("check")

        self.assertEqual(status, 1, output)
        self.assertIn("MISSING-BRANCH ultron/ultron/mainline", output)
        self.assertIn("MISSING-BRANCH peer/ultron/mainline", output)

    def test_apply_creates_branch_at_base_and_is_idempotent(self):
        self._write_manifest()

        status, output = self.invoke("apply")
        self.assertEqual(status, 0, output)
        self.assertIn("CREATED-BRANCH ultron/ultron/mainline", output)
        self.assertIn("CREATED-BRANCH peer/ultron/mainline", output)
        self.assertTrue(self._branch_exists(self.source, "ultron/mainline"))
        self.assertTrue(self._branch_exists(self.peer, "ultron/mainline"))
        self.assertEqual(
            self._rev(self.source, "ultron/mainline"), self._rev(self.source, "main")
        )
        self.assertEqual(
            self._rev(self.peer, "ultron/mainline"), self._rev(self.peer, "master")
        )

        status, output = self.invoke("apply")
        self.assertEqual(status, 0, output)
        self.assertNotIn("CREATED-BRANCH", output)
        self.assertIn("APPLY OK: 0 path(s) updated", output)
        self.assertEqual(self.invoke("check")[0], 0)

    def test_existing_mainline_branch_is_left_untouched(self):
        self._write_manifest()
        self.assertEqual(self.invoke("apply")[0], 0)
        original = self._rev(self.peer, "ultron/mainline")

        # Advance the base branch after the mainline branch was created.
        (self.peer / "advance.txt").write_text("more\n")
        self._git(self.peer, "add", "advance.txt")
        self._git(self.peer, "commit", "-q", "-m", "advance")
        self.assertNotEqual(self._rev(self.peer, "master"), original)

        status, output = self.invoke("check")
        self.assertEqual(status, 0, output)
        self.assertNotIn("MISSING-BRANCH", output)

        status, output = self.invoke("apply")
        self.assertEqual(status, 0, output)
        self.assertNotIn("CREATED-BRANCH", output)
        self.assertEqual(self._rev(self.peer, "ultron/mainline"), original)

    def test_rejects_bad_mainline_branch(self):
        self.manifest["mainline_branch"] = "  "
        self._write_manifest()

        status, output = self.invoke("check")

        self.assertEqual(status, 2)
        self.assertIn("mainline_branch", output)

    def test_rejects_bad_base_branch(self):
        self.manifest["repositories"][1]["base_branch"] = "bad..branch"
        self._write_manifest()

        status, output = self.invoke("check")

        self.assertEqual(status, 2)
        self.assertIn("base_branch", output)

    def test_apply_errors_when_base_branch_missing(self):
        self.manifest["repositories"][1]["base_branch"] = "nonexistent"
        self._write_manifest()

        status, output = self.invoke("apply")

        self.assertEqual(status, 2)
        self.assertIn("base branch", output)
        self.assertFalse(self._branch_exists(self.peer, "ultron/mainline"))


if __name__ == "__main__":
    unittest.main()
