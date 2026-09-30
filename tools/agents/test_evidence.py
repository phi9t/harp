"""evidence.py against throwaway worker runs and git repos."""

from __future__ import annotations

import hashlib
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import evidence  # noqa: E402


def run(cwd, *args):
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True, text=True).stdout


def commit(cwd, message):
    return run(cwd, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", message)


def canonical_bytes(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def record_path(out_dir, digest):
    return Path(out_dir) / f"{digest.removeprefix('sha256:')}.json"


def claims_path(out_dir, digest):
    return Path(out_dir) / f"{digest.removeprefix('sha256:')}.claims.jsonl"


def worktree_diff_digest(out_dir, digest):
    return evidence.load(digest, out_dir=out_dir)["fields"]["worktree_diff_digest"]["value"]


class EvidenceTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.case = 0
        self.old_env = os.environ.get("CORENIUS_EVIDENCE")
        os.environ.pop("CORENIUS_EVIDENCE", None)

    def tearDown(self):
        if self.old_env is None:
            os.environ.pop("CORENIUS_EVIDENCE", None)
        else:
            os.environ["CORENIUS_EVIDENCE"] = self.old_env
        self.tmp.cleanup()

    def make_worker_run(self, dirty=True):
        self.case += 1
        home = self.root / f"case-{self.case}"
        repo = home / "repo"
        repo.mkdir(parents=True)
        run(repo, "init", "-q", "-b", "main")
        (repo / ".gitignore").write_text(".corenius/\n")
        (repo / "README.md").write_text("base\n")
        run(repo, "add", ".gitignore", "README.md")
        commit(repo, "init")
        base_sha = run(repo, "rev-parse", "HEAD").strip()
        if dirty:
            (repo / "README.md").write_text("worker change\n")

        run_dir = home / "run"
        run_dir.mkdir()
        run_json = {
            "name": "ticket-05",
            "run_dir": str(run_dir),
            "worktree": str(repo),
            "branch": "worker/ticket-05",
            "base_sha": base_sha,
            "allowed": ["tools/agents/evidence.py"],
            "protected": [],
            "forbidden": [],
            "model": "GPT-5.5",
            "effort": "xhigh",
            "sandbox": "danger-full-access (capsule)",
            "cap_seconds": 2700,
            "stall_seconds": 300,
            "pid": 12345,
            "started": "2026-09-30T00:00:00Z",
            "started_epoch": 1.5,
        }
        verdict_json = {
            "verdict": "CLEAN",
            "exit_code": 0,
            "elapsed_seconds": 7,
            "events": 3,
            "finished": "2026-09-30T00:00:07Z",
        }
        (run_dir / "run.json").write_text(json.dumps(run_json, indent=2) + "\n")
        (run_dir / "prompt.md").write_text("Build evidence.\n")
        (run_dir / "events.jsonl").write_text('{"type":"thread.started"}\n{"type":"turn.completed"}\n')
        (run_dir / "verdict.json").write_text(json.dumps(verdict_json, indent=2) + "\n")
        (run_dir / "last-message.md").write_text("done\n")
        (run_dir / "stderr.log").write_text("")
        return repo, run_dir, home / "evidence"

    def test_record_then_verify_and_digest_is_canonical_without_digest(self):
        repo, run_dir, out_dir = self.make_worker_run()
        digest = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)

        self.assertEqual(evidence.verify(digest, out_dir=out_dir), [])
        raw = json.loads(record_path(out_dir, digest).read_text())
        without_digest = dict(raw)
        without_digest.pop("digest")
        expected = "sha256:" + hashlib.sha256(canonical_bytes(without_digest)).hexdigest()
        self.assertEqual(digest, expected)

    def test_record_file_tamper_fails_verify_and_names_record_digest(self):
        repo, run_dir, out_dir = self.make_worker_run()
        digest = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        path = record_path(out_dir, digest)
        record = json.loads(path.read_text())
        record["fields"]["model"]["value"] = "other-model"
        path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")

        problems = evidence.verify(digest, out_dir=out_dir)
        self.assertIn("record digest mismatch", "\n".join(problems))

    def test_recomputable_input_tamper_fails_verify_and_names_field(self):
        cases = [
            ("prompt.md", "changed prompt\n", "prompt_digest"),
            ("events.jsonl", '{"type":"tampered"}\n', "event_stream_digest"),
        ]
        for filename, content, field in cases:
            with self.subTest(filename=filename):
                repo, run_dir, out_dir = self.make_worker_run()
                digest = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
                (run_dir / filename).write_text(content)
                problems = evidence.verify(digest, out_dir=out_dir)
                self.assertIn(field, "\n".join(problems))

        repo, run_dir, out_dir = self.make_worker_run()
        digest = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        (repo / "README.md").write_text("different diff\n")
        problems = evidence.verify(digest, out_dir=out_dir)
        self.assertIn("worktree_diff_digest", "\n".join(problems))

    def test_untracked_only_change_is_reflected_in_worktree_digest(self):
        repo, run_dir, out_dir = self.make_worker_run(dirty=False)
        clean = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        empty_digest = "sha256:" + hashlib.sha256(b"").hexdigest()
        self.assertEqual(worktree_diff_digest(out_dir, clean), empty_digest)

        (repo / "worker-output.txt").write_text("new worker output\n")
        (repo / "payload.bin").write_bytes(b"\x00worker\xffoutput\n")
        (repo / "payload.link").symlink_to("payload.bin")
        dirty = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)

        self.assertNotEqual(worktree_diff_digest(out_dir, dirty), worktree_diff_digest(out_dir, clean))
        self.assertNotEqual(worktree_diff_digest(out_dir, dirty), empty_digest)
        self.assertEqual(evidence.verify(dirty, out_dir=out_dir), [])

    def test_verify_fails_when_untracked_file_is_edited_added_or_deleted(self):
        cases = ("edited", "added", "deleted")
        for mutation in cases:
            with self.subTest(mutation=mutation):
                repo, run_dir, out_dir = self.make_worker_run(dirty=False)
                if mutation in {"edited", "deleted"}:
                    (repo / "worker-output.txt").write_text("original\n")
                digest = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)

                if mutation == "edited":
                    (repo / "worker-output.txt").write_text("changed\n")
                elif mutation == "added":
                    (repo / "worker-output.txt").write_text("added after record\n")
                else:
                    (repo / "worker-output.txt").unlink()

                problems = evidence.verify(digest, out_dir=out_dir)
                self.assertIn("worktree_diff_digest", "\n".join(problems))

    def test_verify_fails_when_tracked_file_is_edited_after_record(self):
        repo, run_dir, out_dir = self.make_worker_run(dirty=False)
        digest = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)

        (repo / "README.md").write_text("tracked edit after record\n")

        problems = evidence.verify(digest, out_dir=out_dir)
        self.assertIn("worktree_diff_digest", "\n".join(problems))

    def test_worktree_digest_includes_staged_changes_and_tracked_deletions(self):
        empty_digest = "sha256:" + hashlib.sha256(b"").hexdigest()

        repo, run_dir, out_dir = self.make_worker_run(dirty=False)
        (repo / "README.md").write_text("staged tracked edit\n")
        run(repo, "add", "README.md")
        staged = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        self.assertNotEqual(worktree_diff_digest(out_dir, staged), empty_digest)
        self.assertEqual(evidence.verify(staged, out_dir=out_dir), [])

        repo, run_dir, out_dir = self.make_worker_run(dirty=False)
        (repo / "README.md").unlink()
        deleted = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        self.assertNotEqual(worktree_diff_digest(out_dir, deleted), empty_digest)
        self.assertEqual(evidence.verify(deleted, out_dir=out_dir), [])

    def test_ignored_files_do_not_affect_worktree_digest(self):
        repo, run_dir, out_dir = self.make_worker_run(dirty=False)
        (repo / ".git" / "info" / "exclude").write_text("ignored-output.txt\n")
        before = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)

        (repo / "ignored-output.txt").write_text("ignored worker output\n")
        after_ignored = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        self.assertEqual(worktree_diff_digest(out_dir, before), worktree_diff_digest(out_dir, after_ignored))

        (repo / "visible-output.txt").write_text("non-ignored worker output\n")
        after_visible = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        self.assertNotEqual(worktree_diff_digest(out_dir, after_ignored), worktree_diff_digest(out_dir, after_visible))

    def test_worktree_digest_is_stable_for_unchanged_untracked_tree(self):
        repo, run_dir, out_dir = self.make_worker_run(dirty=False)
        (repo / "worker-output.txt").write_text("same output\n")

        first = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        second = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        empty_digest = "sha256:" + hashlib.sha256(b"").hexdigest()

        self.assertNotEqual(first, second)
        self.assertEqual(worktree_diff_digest(out_dir, first), worktree_diff_digest(out_dir, second))
        self.assertNotEqual(worktree_diff_digest(out_dir, first), empty_digest)

    def test_claim_sidecar_tamper_and_middle_line_deletion_fail_verify(self):
        repo, run_dir, out_dir = self.make_worker_run()
        digest = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        for index in range(3):
            evidence.append_claim_outcome(
                digest,
                f"claim-{index}",
                True,
                f"python3 check-{index}.py",
                "sha256:" + "0" * 64,
                out_dir=out_dir,
            )
        self.assertEqual(evidence.verify(digest, out_dir=out_dir), [])

        path = claims_path(out_dir, digest)
        lines = path.read_text().splitlines()
        changed = list(lines)
        changed[1] = changed[1].replace("claim-1", "claim-x")
        path.write_text("\n".join(changed) + "\n")
        self.assertIn("claims chain broken", "\n".join(evidence.verify(digest, out_dir=out_dir)))

        repo, run_dir, out_dir = self.make_worker_run()
        digest = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        for index in range(3):
            evidence.append_claim_outcome(
                digest,
                f"claim-{index}",
                True,
                "python3 check.py",
                "sha256:" + "1" * 64,
                out_dir=out_dir,
            )
        path = claims_path(out_dir, digest)
        lines = path.read_text().splitlines()
        path.write_text("\n".join([lines[0], lines[2]]) + "\n")
        self.assertIn("claims chain broken", "\n".join(evidence.verify(digest, out_dir=out_dir)))

    def test_identical_runs_have_distinct_digests(self):
        repo, run_dir, out_dir = self.make_worker_run()
        first = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        second = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        self.assertNotEqual(first, second)
        self.assertEqual(evidence.verify(first, out_dir=out_dir), [])
        self.assertEqual(evidence.verify(second, out_dir=out_dir), [])

    def test_measured_and_declared_fields_are_separate(self):
        repo, run_dir, out_dir = self.make_worker_run()
        digest = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        record = evidence.load(digest, out_dir=out_dir)

        for name, field in record["fields"].items():
            with self.subTest(field=name):
                self.assertEqual(set(field), {"value", "provenance", "source"})
                self.assertIn(field["provenance"], {"measured", "declared"})
        self.assertEqual(record["fields"]["model"]["value"], "GPT-5.5")
        self.assertEqual(record["fields"]["model"]["provenance"], "declared")
        self.assertEqual(record["fields"]["prompt_digest"]["provenance"], "measured")
        self.assertIn("run_json_base_sha", record["fields"])
        self.assertEqual(record["fields"]["base_sha"]["provenance"], "measured")
        self.assertEqual(record["fields"]["run_json_base_sha"]["provenance"], "declared")

    def test_append_claim_outcome_is_append_only_hash_chained_and_keeps_record(self):
        repo, run_dir, out_dir = self.make_worker_run()
        digest = evidence.record_run(run_dir, repo=repo, out_dir=out_dir)
        before = record_path(out_dir, digest).read_bytes()
        evidence.append_claim_outcome(digest, "claim-1", True, "python3 test.py", "sha256:" + "2" * 64, out_dir=out_dir)
        evidence.append_claim_outcome(digest, "claim-2", False, "sh check.sh", "sha256:" + "3" * 64, out_dir=out_dir)
        after = record_path(out_dir, digest).read_bytes()
        lines = claims_path(out_dir, digest).read_bytes().splitlines()

        self.assertEqual(before, after)
        self.assertEqual(len(lines), 2)
        first = json.loads(lines[0].decode("utf-8"))
        second = json.loads(lines[1].decode("utf-8"))
        self.assertEqual(first["prev"], hashlib.sha256(digest.encode("utf-8")).hexdigest())
        self.assertEqual(second["prev"], hashlib.sha256(lines[0]).hexdigest())

    def test_environment_out_dir_and_default_directory_selection(self):
        repo, run_dir, out_dir = self.make_worker_run()
        env_dir = out_dir / "from-env"
        explicit_dir = out_dir / "explicit"
        os.environ["CORENIUS_EVIDENCE"] = str(env_dir)

        env_digest = evidence.record_run(run_dir, repo=repo)
        explicit_digest = evidence.record_run(run_dir, repo=repo, out_dir=explicit_dir)
        self.assertTrue(record_path(env_dir, env_digest).exists())
        self.assertTrue(record_path(explicit_dir, explicit_digest).exists())
        self.assertFalse(record_path(env_dir, explicit_digest).exists())

        os.environ.pop("CORENIUS_EVIDENCE", None)
        default_digest = evidence.record_run(run_dir, repo=repo)
        self.assertTrue(record_path(repo / ".corenius" / "evidence", default_digest).exists())
        self.assertIn(".corenius/", (repo / ".gitignore").read_text().splitlines())
        project_gitignore = Path(__file__).resolve().parents[2] / ".gitignore"
        self.assertIn(".corenius/", project_gitignore.read_text().splitlines())

    def test_cli_record_verify_and_show(self):
        repo, run_dir, out_dir = self.make_worker_run()
        os.environ["CORENIUS_EVIDENCE"] = str(out_dir)
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            code = evidence.main(["record", str(run_dir), "--repo", str(repo)])
        self.assertEqual(code, 0, err.getvalue())
        digest = out.getvalue().strip()

        out = io.StringIO()
        with redirect_stdout(out):
            code = evidence.main(["verify", digest])
        self.assertEqual(code, 0, out.getvalue())

        out = io.StringIO()
        with redirect_stdout(out):
            code = evidence.main(["show", digest])
        self.assertEqual(code, 0)
        self.assertIn(digest, out.getvalue())


if __name__ == "__main__":
    unittest.main()
