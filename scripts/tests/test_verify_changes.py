"""Exercise changed-surface dispatch through disposable Git repositories."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "verify_changes.py"
spec = importlib.util.spec_from_file_location("verify_changes", SCRIPT)
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)


class VerificationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
        self.git("init", "-q")
        self.git("config", "user.name", "Verification fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("config", "core.hooksPath", "/dev/null")
        self.write("AGENTS.md", "# Guidance\n")
        self.write("scripts/verify_changes.py", SCRIPT.read_text())
        self.git("add", "AGENTS.md", "scripts/verify_changes.py")
        self.git("commit", "-qm", "fixture")

    def write(self, name, content):
        p = self.root / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content)
        return p

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, env=self.env).decode().strip()

    def run_gate(self, *args):
        return subprocess.run([sys.executable, str(SCRIPT), *args], cwd=self.root,
                              env=self.env, capture_output=True, text=True)

    def selected(self, *args):
        result = self.run_gate("--plan", *args)
        self.assertEqual(result.returncode, 0, result.stderr)
        return json.loads(result.stdout)

    def test_docs_gate_runs_without_lean_react_or_mise(self):
        self.write("AGENTS.md", "# Updated guidance\n")
        result = self.run_gate()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual({j["surface"] for j in self.selected()["checks"]}, {"diff", "docs"})

    def test_corenius_import_selects_its_preservation_gate(self):
        self.write("corenius/controller/src/lib.rs", "// retained upstream source\n")
        self.write("corenius/docs/historical.md", "[Historical locator](missing-at-capture.md)\n")
        jobs = self.selected()["checks"]
        self.assertEqual({j["surface"] for j in jobs}, {"diff", "corenius-import"})
        self.assertIn(["python3", "scripts/verify_corenius_import.py"], [j["command"] for j in jobs])

    def test_durable_runtime_selects_process_lifecycle_checks(self):
        self.write("harp_durable/__main__.py", "# runtime change\n")
        jobs = self.selected()["checks"]
        self.assertEqual({j["surface"] for j in jobs}, {"diff", "durable-runtime"})
        self.assertIn(["env", "PYTHONPATH=corenius:.", "PYTHONDONTWRITEBYTECODE=1", "python3", "-m", "unittest", "discover", "-s", "harp_durable", "-t", ".", "-p", "test_*.py"], [j["command"] for j in jobs])

    def test_staged_untracked_and_both_rename_surfaces_are_selected(self):
        (self.root / "docs").mkdir()
        self.git("mv", "AGENTS.md", "docs/renamed.md")
        self.write("formalization/lean/TrainingDynamics/New.lean", "theorem x : True := by trivial\n")
        result = self.selected()
        self.assertIn("AGENTS.md", result["paths"])
        self.assertIn("docs/renamed.md", result["paths"])
        self.assertIn("formalization/lean/TrainingDynamics/New.lean", result["paths"])
        self.assertIn(["mise", "run", "lean-training"], [j["command"] for j in result["checks"]])

    def test_clean_candidate_and_explicit_base_include_committed_changes(self):
        before = self.git("rev-parse", "HEAD")
        self.write("docs/agents/policy.md", "# Policy\n")
        self.git("add", "docs/agents/policy.md")
        self.git("commit", "-qm", "policy")
        self.assertIn("docs/agents/policy.md", self.selected()["paths"])
        self.assertEqual(self.selected("--base", before)["base"], before)

    def test_corpus_changes_do_not_select_proof_builds(self):
        self.write("content/example.json", "{}\n")
        jobs = self.selected()["checks"]
        self.assertEqual({j["surface"] for j in jobs}, {"diff", "corpus"})
        self.assertFalse(any("lean-all" in j["command"] for j in jobs))

    def test_affected_failure_blocks_gate_and_does_not_run_later_check(self):
        self.write("content/example.json", "{}\n")
        self.write("AGENTS.md", "# New guidance\n")
        tools = self.root / "fake-tools"
        tools.mkdir()
        mise = tools / "cargo"
        mise.write_text("#!/bin/sh\nexit 7\n")
        mise.chmod(0o755)
        self.env["PATH"] = str(tools) + os.pathsep + self.env["PATH"]
        # Fake executables are test inputs, not candidate paths.
        self.write(".gitignore", "fake-tools/\n")
        result = self.run_gate()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("cargo", result.stderr)
        self.assertNotIn("[verify:docs]", result.stdout)

    def test_changed_skill_frontmatter_and_links_are_checked(self):
        self.write(".agents/skills/example/SKILL.md", "---\nname: example\ndescription: Example\n---\n[missing](missing.md)\n")
        result = self.run_gate()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("broken local link", result.stderr)
        self.write(".agents/skills/example/missing.md", "# Present\n")
        self.assertEqual(self.run_gate().returncode, 0)

    def test_indented_fence_links_are_examples_but_prose_links_are_checked(self):
        self.write("docs/guide.md", "# Guide\n\n  ```text\n  [example](absent.md)\n  ```\n`[inline example](absent.md)`\n")
        self.assertEqual(self.run_gate().returncode, 0)
        self.write("docs/guide.md", "# Guide\n\n[real link](absent.md)\n")
        result = self.run_gate()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("broken local link", result.stderr)

    def test_unknown_surface_and_invalid_base_fail_before_execution(self):
        self.write("unknown/file.bin", "data")
        result = self.run_gate()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("no check mapped", result.stderr)
        self.assertNotEqual(self.run_gate("--base", "missing-ref").returncode, 0)

    def test_shared_rust_contract_selects_consumers_and_executes_the_checks(self):
        self.write("crates/harp-contracts/src/lib.rs", "pub struct Contract;\n")
        packages = [
            {"name": "harp-contracts", "id": "contracts", "manifest_path": str(self.root / "crates/harp-contracts/Cargo.toml"), "dependencies": []},
            {"name": "harp", "id": "harp", "manifest_path": str(self.root / "crates/harp/Cargo.toml"), "dependencies": [{"name": "harp-contracts"}]},
        ]
        metadata = self.write("fake-tools/metadata.json", json.dumps({"packages": packages, "workspace_members": ["contracts", "harp"]}))
        log = self.root / "fake-tools/checks.log"
        cargo = self.write("fake-tools/cargo", '#!/bin/sh\ncase "$1" in\nmetadata) cat "$FIXTURE_METADATA" ;;\ntest) printf "%s\\n" "$*" >> "$CHECKS_LOG" ;;\n*) exit 9 ;;\nesac\n')
        cargo.chmod(0o755)
        self.write(".gitignore", "fake-tools/\n")
        self.env.update(PATH=str(cargo.parent) + os.pathsep + self.env["PATH"], FIXTURE_METADATA=str(metadata), CHECKS_LOG=str(log))
        jobs = self.selected()["checks"]
        self.assertEqual({j["command"][j["command"].index("-p") + 1] for j in jobs if "-p" in j["command"]}, {"harp", "harp-contracts"})
        self.assertEqual(self.run_gate().returncode, 0)
        self.assertIn("-p harp --lib", log.read_text())
        self.assertIn("-p harp-contracts", log.read_text())

    def test_portable_profile_changes_select_real_asset_validation(self):
        self.write("tools/obsidian/profile/bookmarks.json", "[]\n")
        jobs = self.selected()["checks"]
        self.assertIn(["python3", "scripts/validate_obsidian_assets.py"], [job["command"] for job in jobs])

    def test_retired_article_tools_and_projections_select_repository_check(self):
        for relative in ("reference/weng-source-cards.html", "labs/meta_harness_trae/run.py", "scripts/build_weng_course.py"):
            self.write(relative, "old projection\n")
        self.git("add", ".")
        self.git("commit", "-qm", "old article projections")
        for relative in ("reference/weng-source-cards.html", "labs/meta_harness_trae/run.py", "scripts/build_weng_course.py"):
            (self.root / relative).unlink()
        jobs = self.selected()["checks"]
        self.assertIn(["mise", "run", "verify-repository"], [job["command"] for job in jobs])

    def test_deleted_document_does_not_require_the_deleted_file(self):
        (self.root / "AGENTS.md").unlink()
        self.assertEqual(self.run_gate().returncode, 0)


if __name__ == "__main__":
    unittest.main()
