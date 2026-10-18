"""effort.py against a throwaway git repo: the dehydrate/rehydrate round trip."""

from __future__ import annotations

import io
import json
import shutil
import subprocess
import sys
import tempfile
import textwrap
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import effort  # noqa: E402

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


def run(cwd, *args):
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True, text=True).stdout


class EffortTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.repo = root / "proj"
        self.repo.mkdir()
        run(self.repo, "init", "-q", "-b", "main")
        run(self.repo, "config", "user.email", "a@b")
        run(self.repo, "config", "user.name", "a")
        (self.repo / ".gitignore").write_text(".scratch/\n")
        (self.repo / "README").write_text("hi\n")
        run(self.repo, "add", ".")
        run(self.repo, "commit", "-qm", "init")
        launched = self.call(["--repo", str(self.repo), "new", "demo"])
        self.wt = Path(json.loads(launched)["worktree"])
        run(self.wt, "config", "user.email", "a@b")
        run(self.wt, "config", "user.name", "a")

    def tearDown(self):
        self.tmp.cleanup()

    def call(self, argv):
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            self.code = effort.main(argv)
        self.err = err.getvalue()
        return out.getvalue()

    def fill_state(self):
        state = self.wt / "docs/efforts/demo/STATE.md"
        text = state.read_text()
        text = text.replace("## Where we are\n\n_TODO_", "## Where we are\n\nStep one done.")
        text = text.replace("## Next actions\n\n_TODO_", "## Next actions\n\n1. Do step two.")
        state.write_text(text)

    def second_worktree(self):
        other = self.repo.parent / "proj-second"
        run(self.repo, "worktree", "add", "-q", "-b", "work/second", str(other), "main")
        (other / "f.txt").write_text("x\n")
        run(other, "add", "f.txt")
        run(other, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "work")
        return other

    def write_legacy_effort(self, root: Path, slug: str) -> None:
        effort_root = root / "docs" / "efforts" / slug
        effort_root.mkdir(parents=True, exist_ok=True)
        (effort_root / "EFFORT.md").write_text(
            f"# Effort: {slug}\n\n"
            "Status: active\n"
            "Base: main\n"
            "Lead: lead\n"
        )
        (effort_root / "STATE.md").write_text(
            textwrap.dedent(
                f"""\
                # State: {slug}

                {effort.START}
                ```json
                {{
                  "base_branch": "main",
                  "base_commit": "{run(self.repo, 'rev-parse', 'main').strip()}",
                  "dehydrated": "2026-10-05T00:00:00Z",
                  "effort": "{slug}",
                  "runs": [],
                  "worktrees": []
                }}
                ```
                {effort.END}

                ## Where we are

                Legacy effort exists in many checkouts.

                ## Next actions

                Keep status usable.
                """
            )
        )

    def test_new_creates_durable_and_scratch(self):
        self.assertTrue((self.wt / "docs/efforts/demo/EFFORT.md").exists())
        self.assertTrue((self.repo / ".scratch/demo").is_dir())
        self.assertIn("Integration branch: effort/demo", (self.wt / "docs/efforts/demo/EFFORT.md").read_text())

    def test_move_records_creates_record_branch_and_removes_effort_from_code_branch(self):
        run(self.wt, "add", "docs/efforts/demo")
        run(self.wt, "commit", "-qm", "start effort")

        out = self.call(["--repo", str(self.wt), "move-records", "demo"])

        self.assertEqual(self.code, effort.OK, self.err)
        moved = json.loads(out)
        record_worktree = Path(moved["record_worktree"])
        self.assertEqual(moved["record_branch"], "effort/demo-records")
        self.assertTrue((record_worktree / "docs/efforts/demo/EFFORT.md").exists())
        self.assertIn("Record branch: effort/demo-records", (record_worktree / "docs/efforts/demo/EFFORT.md").read_text())
        self.assertFalse((self.wt / "docs/efforts/demo").exists())
        self.assertEqual(run(record_worktree, "branch", "--show-current").strip(), "effort/demo-records")
        self.assertEqual(run(self.wt, "branch", "--show-current").strip(), "effort/demo")
        self.assertEqual(run(self.wt, "status", "--porcelain", "--untracked-files=all"), "")
        self.assertEqual(run(record_worktree, "status", "--porcelain", "--untracked-files=all"), "")

    def test_move_records_rerun_finishes_after_record_branch_commit_interruption(self):
        run(self.wt, "add", "docs/efforts/demo")
        run(self.wt, "commit", "-qm", "start effort")
        record_worktree = self.repo.parent / "proj-demo-records"
        run(self.wt, "worktree", "add", "-q", "-b", "effort/demo-records", str(record_worktree), "HEAD")
        effort_file = record_worktree / "docs/efforts/demo/EFFORT.md"
        effort_file.write_text(effort.append_record_branch(effort_file.read_text(), "effort/demo-records"))
        run(record_worktree, "add", "docs/efforts/demo/EFFORT.md")
        run(record_worktree, "commit", "-qm", "Move demo records to record branch")

        out = self.call(["--repo", str(self.wt), "move-records", "demo"])

        self.assertEqual(self.code, effort.OK, self.err)
        moved = json.loads(out)
        self.assertEqual(moved["record_worktree"], str(record_worktree.resolve()))
        self.assertTrue((record_worktree / "docs/efforts/demo/EFFORT.md").exists())
        self.assertFalse((self.wt / "docs/efforts/demo").exists())
        self.assertEqual(run(self.wt, "status", "--porcelain", "--untracked-files=all"), "")
        self.assertEqual(run(record_worktree, "status", "--porcelain", "--untracked-files=all"), "")

    def test_dehydrate_requires_human_part(self):
        self.call(["--repo", str(self.wt), "dehydrate", "demo"])
        self.assertEqual(self.code, effort.ERROR)
        self.assertIn("Where we are", self.err)

    def test_snapshot_preserves_human_text_dirty_files_index_and_refs(self):
        self.fill_state()
        state = self.wt / "docs/efforts/demo/STATE.md"
        human = effort.split_state(state.read_text())[1]
        (self.wt / "README").write_text("staged\n")
        run(self.wt, "add", "README")
        (self.wt / "README").write_text("unstaged\n")
        (self.wt / "untracked file").write_bytes(b"\x00binary")
        evidence = self.wt / "docs/efforts/demo/evidence/check.txt"
        evidence.write_text("focused checks passed\n")
        index = Path(run(self.wt, "rev-parse", "--path-format=absolute", "--git-path", "index").strip())
        before = (index.read_bytes(), run(self.wt, "rev-parse", "HEAD"), run(self.wt, "show-ref"))

        out = self.call(["--repo", str(self.wt), "snapshot", "demo", "--evidence", str(evidence)])

        self.assertEqual(self.code, effort.OK, self.err)
        self.assertEqual(before, (index.read_bytes(), run(self.wt, "rev-parse", "HEAD"), run(self.wt, "show-ref")))
        machine, after_human = effort.split_state(state.read_text())
        self.assertEqual(after_human, human)
        self.assertEqual(machine["mode"], "snapshot")
        self.assertEqual(machine["evidence"][0]["path"], str(evidence))
        self.assertEqual(len(machine["evidence"][0]["sha256"]), 64)
        self.assertIn("README", machine["worktrees"][0]["dirty"])
        self.assertIn("untracked file", machine["worktrees"][0]["dirty"])
        self.assertIn("not backed up", out)
        self.assertEqual((self.wt / "README").read_text(), "unstaged\n")
        self.assertEqual((self.wt / "untracked file").read_bytes(), b"\x00binary")

    def test_dirty_worktree_blocks_dehydrate(self):
        self.fill_state()
        other = self.second_worktree()
        (other / "f.txt").write_text("dirty\n")
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--worktree", str(other)])
        self.assertEqual(self.code, effort.ERROR)
        self.assertIn("uncommitted", self.err)

    def test_rehydrate_refuses_to_recreate_missing_dirty_snapshot_worktree(self):
        self.fill_state()
        other = self.second_worktree()
        (other / "f.txt").write_text("uncommitted work\n")
        self.call(["--repo", str(self.wt), "snapshot", "demo", "--worktree", str(other)])
        self.assertEqual(self.code, effort.OK, self.err)
        run(self.repo, "worktree", "remove", "--force", str(other))

        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])

        self.assertEqual(self.code, effort.ERROR, out)
        self.assertIn("dirty contents were not backed up", out)
        self.assertFalse(other.exists())

    def test_snapshot_retains_missing_dirty_worktree_and_evidence_drift(self):
        self.fill_state()
        other = self.second_worktree()
        (other / "f.txt").write_text("uncommitted work\n")
        evidence = self.wt / "docs/efforts/demo/evidence/check.txt"
        evidence.write_text("passed\n")
        self.call(["--repo", str(self.wt), "snapshot", "demo", "--worktree", str(other), "--evidence", str(evidence)])
        run(self.repo, "worktree", "remove", "--force", str(other))
        evidence.write_text("changed after verification\n")
        self.call(["--repo", str(self.wt), "snapshot", "demo"])
        self.assertEqual(self.code, effort.OK, self.err)
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertEqual(self.code, effort.ERROR, out)
        self.assertIn("dirty contents were not backed up", out)
        self.assertIn("evidence changed", out)
        self.assertFalse(other.exists())

    def test_snapshot_refuses_self_referential_evidence_without_writing(self):
        self.fill_state()
        state = self.wt / "docs/efforts/demo/STATE.md"
        before = state.read_bytes()
        self.call(["--repo", str(self.wt), "snapshot", "demo", "--evidence", str(state)])
        self.assertEqual(self.code, effort.ERROR, self.err)
        self.assertEqual(state.read_bytes(), before)

    def test_snapshot_refuses_unknown_missing_worktree_without_writing(self):
        self.fill_state()
        state = self.wt / "docs/efforts/demo/STATE.md"
        before = state.read_bytes()
        self.call(["--repo", str(self.wt), "snapshot", "demo", "--worktree", str(self.repo.parent / "missing")])
        self.assertEqual(self.code, effort.ERROR, self.err)
        self.assertEqual(state.read_bytes(), before)

    def test_round_trip_restores_worktree_and_briefs(self):
        self.fill_state()
        other = self.second_worktree()
        head = run(other, "rev-parse", "HEAD").strip()
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--worktree", str(other)])
        self.assertEqual(self.code, effort.OK, self.err)
        self.assertEqual(run(self.wt, "status", "--porcelain").strip(), "")
        # Lose the worktree and the scratch.
        run(self.repo, "worktree", "remove", "--force", str(other))
        shutil.rmtree(self.repo / ".scratch")
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertEqual(self.code, effort.OK, out)
        self.assertEqual(run(other, "rev-parse", "HEAD").strip(), head)
        self.assertIn("Do step two", out)
        self.assertIn("restored worktree", out)
        self.assertTrue((self.repo / ".scratch/demo").is_dir())

    def test_missing_branch_is_recreated_from_sha(self):
        self.fill_state()
        other = self.second_worktree()
        head = run(other, "rev-parse", "HEAD").strip()
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--worktree", str(other)])
        run(self.repo, "worktree", "remove", "--force", str(other))
        run(self.repo, "branch", "-D", "work/second")
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertIn("recreated branch", out)
        self.assertEqual(run(other, "rev-parse", "HEAD").strip(), head)

    def test_diverged_branch_is_an_error(self):
        self.fill_state()
        other = self.second_worktree()
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--worktree", str(other)])
        run(self.repo, "worktree", "remove", "--force", str(other))
        run(self.repo, "branch", "-f", "work/second", "main")
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertEqual(self.code, effort.ERROR)
        self.assertIn("diverged", out)

    def test_advanced_branch_is_a_warning(self):
        self.fill_state()
        other = self.second_worktree()
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--worktree", str(other)])
        (other / "g.txt").write_text("y\n")
        run(other, "add", "g.txt")
        run(other, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "more")
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertEqual(self.code, effort.WARN, out)

    def test_rewritten_base_is_an_error(self):
        self.fill_state()
        self.call(["--repo", str(self.wt), "dehydrate", "demo"])
        run(self.repo, "commit", "-q", "--amend", "-m", "rewritten")
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertEqual(self.code, effort.ERROR)
        self.assertIn("base commit", out)

    def test_lost_run_logs_keep_verdict(self):
        self.fill_state()
        run_dir = Path(self.tmp.name) / "runs" / "r1"
        run_dir.mkdir(parents=True)
        (run_dir / "verdict.json").write_text(json.dumps({"verdict": "CLEAN", "exit_code": 0}))
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--run", str(run_dir)])
        shutil.rmtree(run_dir)
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertEqual(self.code, effort.WARN)
        self.assertIn("logs lost, recorded verdict CLEAN", out)

    def test_decisions_are_append_only(self):
        self.call(["--repo", str(self.wt), "decide", "demo", "Use tracked efforts"])
        self.assertEqual(self.code, effort.OK)
        self.call(["--repo", str(self.wt), "decide", "demo", "Use tracked efforts"])
        self.assertEqual(self.code, 64)

    def test_status_lists_effort(self):
        out = self.call(["--repo", str(self.wt), "status"])
        self.assertIn('"effort": "demo"', out)
        self.assertIn('"integration_branch": "effort/demo"', out)
        self.assertIn("never dehydrated", out)

    def test_status_reports_mainline_moved_and_landed_candidates(self):
        effort_file = self.wt / "docs/efforts/demo/EFFORT.md"
        if "Integration branch:" not in effort_file.read_text():
            effort_file.write_text(effort_file.read_text().replace("Base: main\n", "Base: main\nIntegration branch: effort/demo\n"))
            run(self.wt, "add", "docs/efforts/demo/EFFORT.md")
            run(self.wt, "commit", "-qm", "record integration branch")
        evidence = self.wt / "docs/efforts/demo/evidence"
        (self.wt / "candidate-01.txt").write_text("one\n")
        run(self.wt, "add", "candidate-01.txt")
        run(self.wt, "commit", "-qm", "candidate 01")
        candidate_01 = run(self.wt, "rev-parse", "HEAD").strip()
        (evidence / "acceptance-01.jsonl").write_text(json.dumps({
            "schema": "corenius.acceptance.v1",
            "effort": "demo",
            "ticket": "01",
            "candidate": candidate_01,
            "accepted": True,
        }) + "\n")
        run(self.wt, "add", "docs/efforts/demo/evidence/acceptance-01.jsonl")
        run(self.wt, "commit", "-qm", "accept 01")
        (self.wt / "candidate-02.txt").write_text("two\n")
        run(self.wt, "add", "candidate-02.txt")
        run(self.wt, "commit", "-qm", "candidate 02")
        candidate_02 = run(self.wt, "rev-parse", "HEAD").strip()
        (evidence / "acceptance-02.jsonl").write_text(json.dumps({
            "schema": "corenius.acceptance.v1",
            "effort": "demo",
            "ticket": "02",
            "candidate": candidate_02,
            "accepted": True,
        }) + "\n")
        run(self.wt, "add", "docs/efforts/demo/evidence/acceptance-02.jsonl")
        run(self.wt, "commit", "-qm", "accept 02")
        (self.repo / "mainline.txt").write_text("main moved\n")
        run(self.repo, "add", "mainline.txt")
        run(self.repo, "commit", "-qm", "main moved")
        before = {ref: run(self.repo, "rev-parse", ref).strip() for ref in ("main", "effort/demo")}

        out = self.call(["--repo", str(self.wt), "status"])

        after = {ref: run(self.repo, "rev-parse", ref).strip() for ref in ("main", "effort/demo")}
        rows = [json.loads(line) for line in out.splitlines()]
        row = next(row for row in rows if row["effort"] == "demo")
        self.assertEqual(after, before)
        self.assertTrue(row["integration_behind_mainline"])
        self.assertEqual(
            row["landed_candidates"],
            [
                {"ticket": "01", "candidate": candidate_01},
                {"ticket": "02", "candidate": candidate_02},
            ],
        )

    def test_status_finds_efforts_from_linked_worktrees(self):
        run(self.repo, "checkout", "-q", "main")

        out = self.call(["--repo", str(self.repo), "status"])

        rows = [json.loads(line) for line in out.splitlines()]
        row = next(row for row in rows if row["effort"] == "demo")
        self.assertEqual(row["base_branch"], "main")
        self.assertEqual(row["integration_branch"], "effort/demo")
        self.assertEqual(row["effort_worktree"], str(self.wt.resolve()))

    def test_status_uses_integration_record_from_every_checkout(self):
        run(self.repo, "checkout", "-q", "main")
        stale = self.repo.parent / "stale-copy"
        run(self.repo, "worktree", "add", "-q", "-b", "agent/stale-copy", str(stale), "effort/demo")
        effort_file = self.wt / "docs/efforts/demo/EFFORT.md"
        effort_file.write_text(effort_file.read_text().replace("Status: active", "Status: reviewing"))
        run(self.wt, "add", "docs/efforts/demo/EFFORT.md")
        run(self.wt, "commit", "-qm", "update effort status")

        outputs = []
        for checkout in (self.repo, self.wt, stale):
            out = self.call(["--repo", str(checkout), "status"])
            self.assertEqual(self.code, effort.OK, (checkout, self.err))
            rows = [json.loads(line) for line in out.splitlines()]
            outputs.append(next(row for row in rows if row["effort"] == "demo"))

        self.assertEqual([row["status"] for row in outputs], ["reviewing", "reviewing", "reviewing"])
        self.assertEqual({row["effort_worktree"] for row in outputs}, {str(self.wt.resolve())})

    def test_status_and_rehydrate_tolerate_many_legacy_effort_copies(self):
        run(self.repo, "checkout", "-q", "main")
        self.write_legacy_effort(self.repo, "legacy")
        run(self.repo, "add", "docs/efforts/legacy")
        run(self.repo, "commit", "-qm", "legacy effort")
        for idx in range(4):
            path = self.repo.parent / f"legacy-copy-{idx}"
            run(self.repo, "worktree", "add", "-q", "-b", f"work/legacy-copy-{idx}", str(path), "main")

        out = self.call(["--repo", str(self.repo), "status"])

        self.assertEqual(self.code, effort.OK, self.err)
        rows = [json.loads(line) for line in out.splitlines()]
        legacy = next(row for row in rows if row["effort"] == "legacy")
        self.assertEqual(legacy["effort_worktree"], str(self.repo.resolve()))
        self.assertNotIn("problem", legacy)

        out = self.call(["--repo", str(self.repo), "rehydrate", "legacy"])

        self.assertIn("# Briefing: legacy", out)
        self.assertNotIn("ambiguous effort record", out + self.err)


if __name__ == "__main__":
    unittest.main()
