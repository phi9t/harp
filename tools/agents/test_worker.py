"""worker.py against a throwaway git repo and fake worker runtimes (no model)."""

from __future__ import annotations

import json
import http.server
import os
import stat
import subprocess
import sys
import tempfile
import textwrap
import threading
import time
import unittest
from unittest import mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import worker  # noqa: E402

FAKE = textwrap.dedent(
    """\
    #!/usr/bin/env python3
    import json, os, shutil, subprocess, sys, tempfile, time, uuid
    args = sys.argv[1:]
    if args[:2] == ["exec", "resume"]:
        session = args[2]
        message = sys.stdin.read()
        if "-o" in args:
            open(args[args.index("-o") + 1], "w").write("resumed\\n")
        log = os.environ.get("FAKE_RESUME_LOG")
        if log:
            with open(log, "a") as fh:
                fh.write(json.dumps({"session": session, "message": message, "argv": args, "cwd": os.getcwd()}) + "\\n")
        print(json.dumps({"type": "thread.started", "thread_id": session}), flush=True)
        sleep = float(os.environ.get("FAKE_RESUME_SLEEP", "0"))
        if sleep:
            time.sleep(sleep)
        print(json.dumps({"type": "turn.completed"}), flush=True)
        if os.environ.get("FAKE_RESUME_EXIT"):
            sys.exit(int(os.environ["FAKE_RESUME_EXIT"]))
        sys.exit(0)
    log = os.environ.get("FAKE_LAUNCH_LOG")
    if log:
        with open(log, "a") as fh:
            fh.write(json.dumps({"argv": args}) + "\\n")
    root = args[args.index("-C") + 1]
    mode = os.environ.get("FAKE_MODE", "clean")
    def emit(event):
        print(json.dumps(event), flush=True)
    emit({"type": "thread.started", "thread_id": os.environ.get("FAKE_SESSION_ID", "trae-session-1")})
    emit({"type": "item.completed", "item": {"id": "i0", "type": "command_execution", "command": "git show HEAD:README"}})
    if mode == "stall":
        time.sleep(30)
    if mode == "forbidden":
        emit({"type": "item.completed", "item": {"id": "i1", "type": "command_execution", "command": "curl http://example.com"}})
    if mode == "network":
        emit({"type": "item.completed", "item": {"id": "i1", "type": "command_execution", "command": "curl http://127.0.0.1:9/fixture"}})
    if mode == "container_build":
        emit({"type": "item.completed", "item": {"id": "i1", "type": "command_execution", "command": "docker build ."}})
    if mode == "shared_cache":
        emit({"type": "item.completed", "item": {"id": "i1", "type": "command_execution", "command": "pip install ."}})
    if mode == "timeout_probe":
        timeout = args[args.index("--shell-tool-timeout") + 1]
        emit({"type": "item.completed", "item": {"id": "i1", "type": "command_execution", "command": "sleep 121"}})
        if timeout != "125s":
            sys.exit(124)
        open(os.path.join(root, "out.md"), "w").write(timeout + "\\n")
        subprocess.run(["git", "-C", root, "add", "out.md"], check=True)
        subprocess.run(["git", "-C", root, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "x"], check=True)
    if mode == "temp_probe":
        forbidden = os.environ.get("CORENIUS_FORBIDDEN_TMP")
        if "TMPDIR" not in os.environ and forbidden:
            tempfile.tempdir = forbidden
        if "TMUX_TMPDIR" not in os.environ and forbidden:
            os.environ["TMUX_TMPDIR"] = os.path.join(forbidden, "tmux")
            os.makedirs(os.environ["TMUX_TMPDIR"], exist_ok=True)
        made = tempfile.mkdtemp(prefix="worker-temp-probe-")
        session = "corenius-worker-temp-" + uuid.uuid4().hex
        tmux_ok = shutil.which("tmux") is not None
        if tmux_ok:
            subprocess.run(["tmux", "new-session", "-d", "-s", session, "true"], check=True)
            subprocess.run(["tmux", "kill-session", "-t", session], check=False, capture_output=True)
        here_doc = os.path.join(os.environ.get("TMPDIR", tempfile.gettempdir()), "here-doc.txt")
        subprocess.run(["sh", "-c", "cat > \\\"$1\\\" <<'EOF'\\nhello\\nEOF", "sh", here_doc], check=True)
        open(os.path.join(root, "out.md"), "w").write(json.dumps({
            "TMPDIR": os.environ.get("TMPDIR"),
            "TMUX_TMPDIR": os.environ.get("TMUX_TMPDIR"),
            "TMPPREFIX": os.environ.get("TMPPREFIX"),
            "mkdtemp": made,
            "here_doc": here_doc,
            "tmux_ok": tmux_ok,
        }, sort_keys=True) + "\\n")
        subprocess.run(["git", "-C", root, "add", "out.md"], check=True)
        subprocess.run(["git", "-C", root, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "x"], check=True)
    if mode == "capsule_fetch":
        import urllib.request
        url = "__FETCH_URL__"
        emit({"type": "item.completed", "item": {"id": "i1", "type": "command_execution", "command": "curl " + url}})
        try:
            urllib.request.urlopen(url, timeout=2).read()
            result = "fetch ok\\n"
        except Exception:
            result = "fetch failed\\n"
        open(os.path.join(root, "out.md"), "w").write(result)
    if mode in ("rm_scratch", "rm_home", "rm_parent", "rm_tracked"):
        cmd = {
            "rm_scratch": 'bash -lc "mkdir -p scratch && rm -rf scratch"',
            "rm_home": "rm -rf ~/somewhere",
            "rm_parent": "rm -rf ../other",
            "rm_tracked": "rm -f README",
        }[mode]
        emit({"type": "item.completed", "item": {"id": "i2", "type": "command_execution", "command": cmd}})
        if mode == "rm_tracked":
            os.remove(os.path.join(root, "README"))
    if mode == "extra":
        open(os.path.join(root, "stray.txt"), "w").write("x")
    marker = os.environ.get("FAKE_MARKER", "ticket 01: ready for review")
    if mode in ("clean", "commit", "move_main", "rewrite_base", "marker_alive", "marker_move",
                "network", "container_build", "shared_cache", "slow_commit"):
        open(os.path.join(root, "out.md"), "w").write("x")
        subprocess.run(["git", "-C", root, "add", "out.md"], check=True)
        message = marker if mode.startswith("marker_") else "x"
        subprocess.run(["git", "-C", root, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", message], check=True)
    if mode == "slow_commit":
        time.sleep(2)
    if mode == "marker_alive":
        time.sleep(30)
    if mode == "marker_dirty":
        open(os.path.join(root, "out.md"), "w").write("x")
        subprocess.run(["git", "-C", root, "add", "out.md"], check=True)
        subprocess.run(["git", "-C", root, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "pre-marker"], check=True)
        open(os.path.join(root, "out.md"), "a").write("dirty\\n")
        subprocess.run(["git", "-C", root, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "--allow-empty", "-qm", marker], check=True)
    if mode == "marker_move":
        time.sleep(1)
        open(os.path.join(root, "out.md"), "a").write("moved\\n")
        subprocess.run(["git", "-C", root, "add", "out.md"], check=True)
        subprocess.run(["git", "-C", root, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "--amend", "-qm", "moved"], check=True)
        time.sleep(30)
    if mode == "move_main":
        head = subprocess.check_output(["git", "-C", root, "rev-parse", "HEAD"], text=True).strip()
        subprocess.run(["git", "-C", root, "branch", "-f", "side", head], check=True)
    if mode == "rewrite_base":
        current = subprocess.check_output(["git", "-C", root, "branch", "--show-current"], text=True).strip()
        subprocess.run(["git", "-C", root, "checkout", "--orphan", "scratch"], check=True)
        subprocess.run(["git", "-C", root, "rm", "-rf", "."], check=True)
        open(os.path.join(root, "out.md"), "w").write("orphan\\n")
        subprocess.run(["git", "-C", root, "add", "out.md"], check=True)
        subprocess.run(["git", "-C", root, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "orphan"], check=True)
        subprocess.run(["git", "-C", root, "branch", "-f", current, "HEAD"], check=True)
        subprocess.run(["git", "-C", root, "checkout", current], check=True)
        subprocess.run(["git", "-C", root, "branch", "-D", "scratch"], check=True)
    if mode in ("dirty", "rm_scratch", "rm_home", "rm_parent", "rm_tracked", "capsule_clean"):
        open(os.path.join(root, "out.md"), "w").write("result\\n")
    if mode == "rm_scratch":
        subprocess.run(["git", "-C", root, "add", "out.md"], check=True)
        subprocess.run(["git", "-C", root, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "x"], check=True)
    if mode == "fail":
        sys.exit(3)
    if mode in ("extra",):
        open(os.path.join(root, "out.md"), "w").write("result\\n")
    time.sleep(0.5)
    emit({"type": "turn.completed"})
    """
)


CODEX_ARCHIVED_SESSION_SAMPLE_ORIGIN = (
    "Sanitized from local Codex session record "
    "~/.codex/archived_sessions/rollout-2026-10-03T05-16-09-01a10030-ef54-7381-85a9-81b7b7e12651.jsonl:12; "
    "kept only as coverage for archived-session records, not codex exec --json."
)

CODEX_EXEC_JSON_SAMPLE_ORIGIN = (
    "Sanitized from lead-captured real codex exec --json run of codex-cli 0.160.0 "
    "on 2026-10-05; ids and usage values are synthetic, command/output keep the observed shape."
)

CODEX_EXEC_JSON_SAMPLE = [
    {"type": "thread.started", "thread_id": "codex-exec-session"},
    {"type": "turn.started"},
    {
        "type": "item.started",
        "item": {
            "id": "cmd-1",
            "type": "command_execution",
            "command": "/bin/bash -lc 'echo corenius-probe'",
            "aggregated_output": "",
            "exit_code": None,
            "status": "in_progress",
        },
    },
    {
        "type": "item.completed",
        "item": {
            "id": "cmd-1",
            "type": "command_execution",
            "command": "/bin/bash -lc 'echo corenius-probe'",
            "aggregated_output": "corenius-probe\n",
            "exit_code": 0,
            "status": "completed",
        },
    },
    {"type": "item.completed", "item": {"id": "msg-1", "type": "agent_message", "text": "done"}},
    {"type": "turn.completed", "usage": {"input_tokens": 11, "cached_input_tokens": 0, "output_tokens": 3}},
]

FAKE_CODEX = textwrap.dedent(
    """\
    #!/usr/bin/env python3
    import json, os, subprocess, sys, time
    args = sys.argv[1:]
    session = os.environ.get("FAKE_CODEX_SESSION_ID", "codex-session-1")
    def emit(event):
        print(json.dumps(event), flush=True)
    if args[:2] == ["exec", "resume"]:
        resumed = args[2]
        message = sys.stdin.read()
        if "-o" in args:
            open(args[args.index("-o") + 1], "w").write("resumed\\n")
        log = os.environ.get("FAKE_RESUME_LOG")
        if log:
            with open(log, "a") as fh:
                fh.write(json.dumps({"session": resumed, "message": message, "argv": args, "cwd": os.getcwd()}) + "\\n")
        emit({"type": "thread.started", "thread_id": resumed})
        sleep = float(os.environ.get("FAKE_RESUME_SLEEP", "0"))
        if sleep:
            time.sleep(sleep)
        emit({"type": "turn.completed", "usage": {"input_tokens": 1, "output_tokens": 1}})
        if os.environ.get("FAKE_RESUME_EXIT"):
            sys.exit(int(os.environ["FAKE_RESUME_EXIT"]))
        sys.exit(0)
    log = os.environ.get("FAKE_LAUNCH_LOG")
    if log:
        with open(log, "a") as fh:
            fh.write(json.dumps({"argv": args}) + "\\n")
    root = args[args.index("-C") + 1]
    mode = os.environ.get("FAKE_MODE", "clean")
    emit({"type": "thread.started", "thread_id": session})
    emit({"type": "turn.started"})
    emit({
        "type": "item.started",
        "item": {
            "id": "cmd-0",
            "type": "command_execution",
            "command": "/bin/bash -lc 'git show HEAD:README'",
            "aggregated_output": "",
            "exit_code": None,
            "status": "in_progress",
        },
    })
    emit({
        "type": "item.completed",
        "item": {
            "id": "cmd-0",
            "type": "command_execution",
            "command": "/bin/bash -lc 'git show HEAD:README'",
            "aggregated_output": "hi\\n",
            "exit_code": 0,
            "status": "completed",
        },
    })
    if mode == "stall":
        time.sleep(30)
    if mode == "forbidden":
        emit({
            "type": "item.started",
            "item": {
                "id": "cmd-forbidden",
                "type": "command_execution",
                "command": "/bin/bash -lc 'curl http://example.com/archive.tgz'",
                "aggregated_output": "",
                "exit_code": None,
                "status": "in_progress",
            },
        })
        emit({
            "type": "item.completed",
            "item": {
                "id": "cmd-forbidden",
                "type": "command_execution",
                "command": "/bin/bash -lc 'curl http://example.com/archive.tgz'",
                "aggregated_output": "",
                "exit_code": 0,
                "status": "completed",
            },
        })
    if mode == "started_only_forbidden":
        emit({
            "type": "item.started",
            "item": {
                "id": "cmd-started-only",
                "type": "command_execution",
                "command": "/bin/bash -lc 'curl http://example.com/archive.tgz'",
                "aggregated_output": "",
                "exit_code": None,
                "status": "in_progress",
            },
        })
    if mode == "archived_forbidden":
        emit({
            "type": "event_msg",
            "payload": {
                "type": "item_completed",
                "thread_id": session,
                "item": {
                    "type": "CommandExecution",
                    "id": "archived-cmd",
                    "command": ["/bin/bash", "-lc", "curl http://example.com/archive.tgz"],
                    "status": "completed",
                },
            },
        })
    if mode == "file_change_add_all":
        emit({
            "type": "item.started",
            "item": {
                "id": "edit-1",
                "type": "file_change",
                "changes": [{"path": os.path.join(root, "out.md"), "kind": "add"}],
                "status": "in_progress",
            },
        })
        open(os.path.join(root, "out.md"), "w").write("dirty through file_change\\n")
        emit({
            "type": "item.completed",
            "item": {
                "id": "edit-1",
                "type": "file_change",
                "changes": [{"path": os.path.join(root, "out.md"), "kind": "add"}],
                "status": "completed",
            },
        })
        emit({
            "type": "item.completed",
            "item": {
                "id": "cmd-add-all",
                "type": "command_execution",
                "command": "/bin/bash -lc 'git add -A'",
                "aggregated_output": "",
                "exit_code": 0,
                "status": "completed",
            },
        })
        subprocess.run(["git", "-C", root, "add", "-A"], check=True)
        time.sleep(30)
    if mode in ("clean", "commit", "forbidden", "started_only_forbidden", "archived_forbidden"):
        open(os.path.join(root, "out.md"), "w").write("x")
        subprocess.run(["git", "-C", root, "add", "out.md"], check=True)
        subprocess.run(["git", "-C", root, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "x"], check=True)
    if mode == "dirty":
        open(os.path.join(root, "out.md"), "w").write("dirty\\n")
    if mode == "fail":
        sys.exit(3)
    time.sleep(0.5)
    emit({"type": "turn.completed", "usage": {"input_tokens": 11, "cached_input_tokens": 0, "output_tokens": 3}})
    """
)


def run(cwd, *args):
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True, text=True).stdout


class WorkerTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.root = root
        self.repo = root / "corenius"
        self.repo.mkdir()
        run(self.repo, "init", "-q", "-b", "main")
        run(self.repo, "config", "user.email", "a@b")
        run(self.repo, "config", "user.name", "a")
        (self.repo / "README").write_text("hi\n")
        run(self.repo, "add", "README")
        run(self.repo, "commit", "-qm", "init")
        run(self.repo, "branch", "side")
        self.fake = root / "traecli"
        self.fake.write_text(FAKE)
        self.fake.chmod(self.fake.stat().st_mode | stat.S_IXUSR)
        self.fake_codex = root / "codex"
        self.fake_codex.write_text(FAKE_CODEX)
        self.fake_codex.chmod(self.fake_codex.stat().st_mode | stat.S_IXUSR)
        self.task = root / "task.md"
        self.task.write_text("Write out.md.\n")
        os.environ["CORENIUS_TRAECLI"] = str(self.fake)
        os.environ["CORENIUS_CODEX"] = str(self.fake_codex)
        os.environ["CORENIUS_RUNS"] = str(root / "runs")
        self.name = 0

    def tearDown(self):
        os.environ.pop("CORENIUS_TRAECLI", None)
        os.environ.pop("CORENIUS_CODEX", None)
        os.environ.pop("CORENIUS_RUNS", None)
        os.environ.pop("FAKE_MODE", None)
        os.environ.pop("FAKE_MARKER", None)
        os.environ.pop("FAKE_SESSION_ID", None)
        os.environ.pop("FAKE_CODEX_SESSION_ID", None)
        os.environ.pop("FAKE_RESUME_LOG", None)
        os.environ.pop("FAKE_RESUME_SLEEP", None)
        os.environ.pop("FAKE_RESUME_EXIT", None)
        os.environ.pop("FAKE_LAUNCH_LOG", None)
        os.environ.pop("CORENIUS_FORBIDDEN_TMP", None)
        os.environ.pop("CORENIUS_CAPSULES", None)
        os.environ.pop("CORENIUS_TRAECLI_TRANSCRIPT_DIR", None)
        os.environ.pop("CORENIUS_CODEX_TRANSCRIPT_DIR", None)
        self.tmp.cleanup()

    def launch(self, mode, *extra):
        os.environ["FAKE_MODE"] = mode
        self.name += 1
        out = self._main(
            ["launch", "--task", str(self.task), "--name", f"t{self.name}", "--allow", "out.md",
             "--repo", str(self.repo), "--stall", "1", *extra]
        )
        return json.loads(out)

    def _main(self, argv):
        import io
        from contextlib import redirect_stdout

        buffer = io.StringIO()
        with redirect_stdout(buffer):
            code = worker.main(argv)
        self.last_code = code
        return buffer.getvalue()

    def supervise(self, launched, **kw):
        out = self._main(["supervise", launched["run_dir"], "--interval", "0.2",
                          "--max-wait", str(kw.get("max_wait", 20))])
        return json.loads(out.strip().splitlines()[-1])

    def status(self, launched, *, update_run_record=False):
        argv = ["status", launched["run_dir"]]
        if update_run_record:
            argv.append("--update-run-record")
        out = self._main(argv)
        return json.loads(out)

    def wait_for_exit(self, launched, timeout=5.0):
        exit_code = Path(launched["run_dir"]) / "exit_code"
        deadline = time.time() + timeout
        while not exit_code.exists() and time.time() < deadline:
            time.sleep(0.05)
        self.assertTrue(exit_code.exists())

    def resume(self, launched, message):
        out = self._main(["resume", launched["run_dir"], "--message", message])
        return json.loads(out)

    def stop(self, launched):
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        try:
            os.killpg(os.getpgid(run_data["pid"]), 15)
        except ProcessLookupError:
            pass

    def tree_listing(self, root: Path) -> list[str]:
        return sorted(str(path.relative_to(root)) for path in root.rglob("*"))

    def append_event_after(self, launched, event: dict, delay: float = 0.05) -> threading.Thread:
        def write_event() -> None:
            time.sleep(delay)
            with (Path(launched["run_dir"]) / "events.jsonl").open("a") as handle:
                handle.write(json.dumps(event) + "\n")

        thread = threading.Thread(target=write_event)
        thread.start()
        return thread

    def test_clean_run(self):
        launched = self.launch("clean")
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertEqual(self.last_code, worker.CLEAN)
        self.assertEqual(verdict["exit_code"], 0)
        self.assertTrue(verdict["ready_for_review"])
        tip = run(self.repo, "rev-parse", launched["branch"]).strip()
        tree = run(self.repo, "rev-parse", f"{tip}^{{tree}}").strip()
        self.assertEqual(verdict["candidate"]["commit"], tip)
        self.assertEqual(verdict["candidate"]["tree"], tree)
        self.assertEqual(verdict["candidate"]["base"], json.loads((Path(launched["run_dir"]) / "run.json").read_text())["base_sha"])
        self.assertEqual(len(verdict["candidate"]["brief_digest"]), 64)
        self.assertTrue((Path(launched["run_dir"]) / "verdict.json").exists())
        self.assertIn("out.md", (Path(launched["worktree"]) / "out.md").name)
        prompt = (Path(launched["run_dir"]) / "prompt.md").read_text()
        self.assertIn("Write out.md.", prompt)
        self.assertIn("out.md", prompt.split("TASK")[0])

    def test_worker_workspace_has_separate_private_git_metadata_and_imports_only_own_branch(self):
        main_before = run(self.repo, "rev-parse", "main").strip()
        side_before = run(self.repo, "rev-parse", "side").strip()

        launched = self.launch("commit")
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())

        git_dir = Path(launched["worktree"]) / ".git"
        self.assertTrue(git_dir.is_file())
        self.assertIn("gitdir:", git_dir.read_text())
        self.assertEqual(run_data["git_dir_layout"], "separate")
        self.assertEqual(Path(run_data["git_dir"]).parent, Path(launched["run_dir"]))
        self.assertTrue(Path(run_data["git_dir"]).is_dir())
        missing_shared = subprocess.run(
            ["git", "-C", str(self.repo), "rev-parse", "--verify", "-q", launched["branch"]],
            capture_output=True,
            text=True,
        )
        self.assertEqual(missing_shared.stdout.strip(), "")
        verdict = self.supervise(launched)

        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertTrue(verdict["ready_for_review"])
        self.assertEqual(run(self.repo, "rev-parse", "main").strip(), main_before)
        self.assertEqual(run(self.repo, "rev-parse", "side").strip(), side_before)
        self.assertEqual(run(self.repo, "rev-parse", launched["branch"]).strip(), verdict["candidate"]["commit"])

    def test_workspace_write_runtimes_get_private_git_dir_as_writable_root(self):
        for runtime in ("traecli", "codex"):
            with self.subTest(runtime=runtime):
                log = Path(self.tmp.name) / f"{runtime}-argv.jsonl"
                os.environ["FAKE_LAUNCH_LOG"] = str(log)
                launched = self.launch("commit", "--runtime", runtime)
                verdict = self.supervise(launched)
                run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
                argv = json.loads(log.read_text().splitlines()[-1])["argv"]
                add_dirs = [argv[i + 1] for i, value in enumerate(argv) if value == "--add-dir"]

                self.assertEqual(verdict["verdict"], "CLEAN", verdict)
                self.assertIn(run_data["git_dir"], add_dirs)

    def test_private_clone_falls_back_when_local_hardlink_crosses_devices(self):
        worktree = Path(self.tmp.name) / "fallback"
        git_dir = Path(self.tmp.name) / "fallback.git"
        original_run = worker.subprocess.run

        def fake_run(argv, *args, **kwargs):
            if argv[:4] == ["git", "clone", "-q", "--local"]:
                return subprocess.CompletedProcess(
                    argv,
                    128,
                    "",
                    "fatal: failed to create link '.git/objects/aa/bb': Invalid cross-device link\n",
                )
            return original_run(argv, *args, **kwargs)

        with mock.patch.object(worker.subprocess, "run", side_effect=fake_run):
            record = worker.create_private_worker_clone(self.repo, worktree, "worker/fallback", "main", git_dir)

        self.assertEqual(record["clone_object_mode"], "copy")
        self.assertEqual(record["git_dir_layout"], "separate")
        self.assertTrue((worktree / ".git").is_file())
        self.assertTrue(git_dir.is_dir())
        self.assertEqual(run(worktree, "rev-parse", "HEAD").strip(), record["base_sha"])

    def test_runtime_option_records_runtime_and_default_stays_traecli(self):
        default = self.launch("clean")
        self.assertEqual(json.loads((Path(default["run_dir"]) / "run.json").read_text())["runtime"], "traecli")
        self.assertEqual(self.supervise(default)["verdict"], "CLEAN")

        codex = self.launch("clean", "--runtime", "codex")
        run_data = json.loads((Path(codex["run_dir"]) / "run.json").read_text())
        self.assertEqual(run_data["runtime"], "codex")
        self.assertIn("Codex", run_data["runtime_observability"]["observed"])
        self.assertEqual(self.supervise(codex)["verdict"], "CLEAN")

    def test_codex_recorded_command_events_drive_forbidden_findings(self):
        self.assertIn("codex-cli 0.160.0", CODEX_EXEC_JSON_SAMPLE_ORIGIN)
        self.assertIn("archived-session records", CODEX_ARCHIVED_SESSION_SAMPLE_ORIGIN)
        self.assertEqual(worker.event_session_id(CODEX_EXEC_JSON_SAMPLE[0], "codex"), "codex-exec-session")
        self.assertEqual(worker.event_stream_commands(CODEX_EXEC_JSON_SAMPLE, "codex"),
                         ["/bin/bash -lc 'echo corenius-probe'"])
        self.assertEqual(
            worker.event_usage(CODEX_EXEC_JSON_SAMPLE[-1], "codex"),
            {"input_tokens": 11, "cached_input_tokens": 0, "output_tokens": 3},
        )
        self.assertEqual(
            worker.event_stream_file_change_paths(
                [{
                    "type": "item.completed",
                    "item": {
                        "id": "edit-1",
                        "type": "file_change",
                        "changes": [{"path": str(self.repo / "out.md"), "kind": "add"}],
                        "status": "completed",
                    },
                }],
                "codex",
                self.repo,
            ),
            ["out.md"],
        )

        events = [
            json.loads(line)
            for line in subprocess.check_output(
            [str(self.fake_codex), "exec", "--json", "-C", str(self.repo), "-o", str(Path(self.tmp.name) / "last"), "-"],
            input="prompt\n",
            text=True,
            env={**os.environ, "FAKE_MODE": "forbidden"},
            ).splitlines()
        ]
        self.assertEqual(
            worker.event_stream_commands(events, "codex"),
            ["/bin/bash -lc 'git show HEAD:README'", "/bin/bash -lc 'curl http://example.com/archive.tgz'"],
        )

        verdict = self.supervise(self.launch("forbidden", "--runtime", "codex", "--network", "off", "--forbid", "curl"))
        self.assertEqual(verdict["verdict"], "VIOLATED", verdict)
        self.assertIn("network-command", json.dumps(verdict["findings"]))
        self.assertIn("forbidden-command", json.dumps(verdict["findings"]))

    def test_codex_file_change_event_accounts_for_dirty_allowed_path(self):
        launched = self.launch("file_change_add_all", "--runtime", "codex")
        try:
            target = Path(launched["worktree"]) / "out.md"
            deadline = time.time() + 5
            while not target.exists() and time.time() < deadline:
                time.sleep(0.1)
            self.assertTrue(target.exists())
            events = Path(launched["run_dir"]) / "events.jsonl"
            deadline = time.time() + 5
            while "cmd-add-all" not in events.read_text() and time.time() < deadline:
                time.sleep(0.1)
            self.assertIn("cmd-add-all", events.read_text())

            status = self.status(launched)

            self.assertTrue(status["alive"], status)
            self.assertNotIn("unaccounted-worktree-change", json.dumps(status["findings"]))
        finally:
            self.stop(launched)

    def test_codex_started_command_without_completion_is_checked_once_on_exit(self):
        verdict = self.supervise(
            self.launch("started_only_forbidden", "--runtime", "codex", "--network", "off", "--forbid", "curl")
        )

        findings = [f for f in verdict["findings"] if f["kind"] == "forbidden-command"]
        self.assertEqual(len(findings), 1, verdict)
        self.assertEqual(verdict["verdict"], "VIOLATED", verdict)

    def test_codex_runtime_usage_is_saved_from_turn_completed(self):
        launched = self.launch("clean", "--runtime", "codex")
        self.assertEqual(self.supervise(launched)["verdict"], "CLEAN")

        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertEqual(run_data["model_usage"], {"input_tokens": 11, "cached_input_tokens": 0, "output_tokens": 3})

    def test_status_records_runtime_session_transcript_path_when_session_id_is_seen(self):
        transcript_root = Path(self.tmp.name) / "trae-transcripts"
        transcript_root.mkdir()
        transcript = transcript_root / "trae-session-1.jsonl"
        transcript.write_text('{"type": "thread.started"}\n')
        os.environ["CORENIUS_TRAECLI_TRANSCRIPT_DIR"] = str(transcript_root)
        launched = self.launch("commit")
        events = Path(launched["run_dir"]) / "events.jsonl"
        deadline = time.time() + 5
        while "trae-session-1" not in events.read_text(errors="replace") and time.time() < deadline:
            time.sleep(0.05)

        status = self.status(launched, update_run_record=True)
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())

        self.assertEqual(status["session_id"], "trae-session-1")
        self.assertEqual(run_data["runtime_transcript"]["path"], str(transcript))
        self.assertEqual(run_data["runtime_transcript"]["session_id"], "trae-session-1")
        self.assertIn("TraeCLI", run_data["runtime_transcript"]["observed"])

    def test_interactive_launch_records_workspace_trust_override(self):
        for runtime in ("traecli", "codex"):
            with self.subTest(runtime=runtime):
                with mock.patch.object(
                    worker,
                    "tmux_interactive_window",
                    return_value={"available": True, "target": f"live-{runtime}:t"},
                ) as tmux_window, mock.patch.object(worker, "read_pid_file", return_value=None):
                    launched = self.launch(
                        "clean",
                        "--runtime",
                        runtime,
                        "--interactive",
                        "--tmux-session",
                        f"live-{runtime}",
                    )
                run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
                expected = f'projects={{"{run_data["worktree"]}"={{trust_level="trusted"}}}}'
                rejected = f'projects."{run_data["worktree"]}".trust_level="trusted"'

                self.assertEqual(run_data["runtime_trust"]["method"], "cli-config")
                self.assertEqual(run_data["runtime_trust"]["config"], expected)
                self.assertIn("-c", run_data["interactive_command"])
                self.assertIn(expected, run_data["interactive_command"])
                self.assertNotIn(rejected, run_data["interactive_command"])
                script = Path(run_data["interactive_script"]).read_text()
                self.assertIn(expected, script)
                self.assertIn(f"export TMPDIR={run_data['temp']['tmpdir']}", script)
                self.assertIn(f"export TMUX_TMPDIR={run_data['temp']['tmux_tmpdir']}", script)
                self.assertIn(f"export TMPPREFIX={run_data['temp']['tmpprefix']}", script)
                self.assertEqual(
                    tmux_window.call_args.args[4],
                    {
                        "TMPDIR": run_data["temp"]["tmpdir"],
                        "TMUX_TMPDIR": run_data["temp"]["tmux_tmpdir"],
                        "TMPPREFIX": run_data["temp"]["tmpprefix"],
                    },
                )

    def test_status_discovers_interactive_transcript_from_workspace_session_meta(self):
        transcript_root = Path(self.tmp.name) / "trae-transcripts"
        transcript_root.mkdir()
        os.environ["CORENIUS_TRAECLI_TRANSCRIPT_DIR"] = str(transcript_root)
        with mock.patch.object(
            worker,
            "tmux_interactive_window",
            return_value={"available": True, "target": "live:01"},
        ), mock.patch.object(worker, "read_pid_file", return_value=None):
            launched = self.launch("clean", "--interactive", "--tmux-session", "live")
        run_json = Path(launched["run_dir"]) / "run.json"
        run_data = json.loads(run_json.read_text())
        run_data.pop("session_id", None)
        run_data.pop("runtime_transcript", None)
        run_data["started_epoch"] = 100.0
        run_json.write_text(json.dumps(run_data, indent=2) + "\n")
        old = transcript_root / "rollout-2026-10-05T11-00-00-old-session.jsonl"
        old.write_text(json.dumps({
            "type": "session_meta",
            "payload": {"session_id": "old-session", "cwd": run_data["worktree"]},
        }) + "\n")
        os.utime(old, (90, 90))
        other = transcript_root / "rollout-2026-10-05T12-00-00-other-session.jsonl"
        other.write_text(json.dumps({
            "type": "session_meta",
            "payload": {"session_id": "other-session", "cwd": str(Path(self.tmp.name) / "other-workspace")},
        }) + "\n")
        os.utime(other, (110, 110))
        transcript = transcript_root / "rollout-2026-10-05T12-00-01-trae-session-2.jsonl"
        transcript.write_text(json.dumps({
            "type": "session_meta",
            "payload": {"session_id": "trae-session-2", "cwd": run_data["worktree"]},
        }) + "\n")
        os.utime(transcript, (120, 120))

        status = self.status(launched, update_run_record=True)
        updated = json.loads(run_json.read_text())

        self.assertEqual(status["session_id"], "trae-session-2")
        self.assertEqual(updated["runtime_transcript"]["session_id"], "trae-session-2")
        self.assertEqual(updated["runtime_transcript"]["path"], str(transcript))

    def test_codex_quiet_time_is_reported_from_its_event_stream(self):
        launched = self.launch("stall", "--runtime", "codex")
        try:
            result = self.supervise(launched, max_wait=3)
            self.assertEqual(result["verdict"], "RUNNING")
            self.assertEqual(self.last_code, worker.RUNNING)
            self.assertIn("stall", json.dumps(result["findings"]))
        finally:
            self.stop(launched)

    def test_handoff_rule_matches_for_traecli_and_codex(self):
        by_runtime = {}
        for runtime in ("traecli", "codex"):
            verdict = self.supervise(self.launch("commit", "--runtime", runtime))
            by_runtime[runtime] = {
                "verdict": verdict["verdict"],
                "ready_for_review": verdict["ready_for_review"],
                "not_ready": verdict["not_ready"],
                "has_candidate": bool(verdict["candidate"]),
            }
        self.assertEqual(by_runtime["codex"], by_runtime["traecli"])

    def test_resume_sends_followup_to_same_session_for_each_runtime(self):
        for runtime, session in (("traecli", "trae-session-resume"), ("codex", "codex-session-resume")):
            os.environ["FAKE_SESSION_ID"] = session
            os.environ["FAKE_CODEX_SESSION_ID"] = session
            resume_log = Path(self.tmp.name) / f"{runtime}-resume.jsonl"
            os.environ["FAKE_RESUME_LOG"] = str(resume_log)
            launched = self.launch("commit", "--runtime", runtime)
            self.assertEqual(self.supervise(launched)["verdict"], "CLEAN")

            receipt = self.resume(launched, "please fix the review finding")
            run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())

            self.assertEqual(receipt["runtime"], runtime)
            self.assertEqual(receipt["session_id"], session)
            self.assertEqual(receipt["return_code"], 0)
            sent = json.loads(resume_log.read_text().splitlines()[-1])
            self.assertEqual(sent["session"], session)
            self.assertEqual(sent["message"], "please fix the review finding")
            self.assertEqual(sent["cwd"], launched["worktree"])
            config_values = [sent["argv"][i + 1] for i, value in enumerate(sent["argv"]) if value in ("-c", "--config")]
            self.assertIn('sandbox_mode="workspace-write"', config_values)
            self.assertIn(f'cwd="{launched["worktree"]}"', config_values)
            self.assertIn(
                f'sandbox_workspace_write.writable_roots=["{run_data["git_dir"]}","{run_data["temp"]["tmpdir"]}"]',
                config_values,
            )

    def test_resume_refreshes_private_clone_integration_branch_from_shared_repo(self):
        os.environ["FAKE_RESUME_LOG"] = str(Path(self.tmp.name) / "resume.jsonl")
        launched = self.launch("commit")
        self.assertEqual(self.supervise(launched)["verdict"], "CLEAN")
        side = Path(self.tmp.name) / "side-worktree"
        run(self.repo, "worktree", "add", "-q", str(side), "side")
        (side / "side.txt").write_text("landed while worker waits\n")
        run(side, "add", "side.txt")
        run(side, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "advance side")
        side_tip = run(self.repo, "rev-parse", "side").strip()
        run_path = Path(launched["run_dir"]) / "run.json"
        run_data = json.loads(run_path.read_text())
        run_data["lead_launch"] = {"integration": "side"}
        run_path.write_text(json.dumps(run_data))

        receipt = self.resume(launched, "rebase onto side")
        run_data = json.loads(run_path.read_text())

        self.assertEqual(receipt["return_code"], 0)
        self.assertEqual(run(Path(launched["worktree"]), "rev-parse", "side").strip(), side_tip)
        self.assertEqual(run_data["branch_baselines"]["side"], side_tip)
        self.assertEqual(run_data["shared_branch_baselines"]["side"], side_tip)
        self.assertNotIn("shared-branch-moved", json.dumps(self.status(launched)["findings"]))

    def test_resume_reports_missing_runtime_session_id(self):
        launched = self.launch("clean")
        run_json = Path(launched["run_dir"]) / "run.json"
        run_data = json.loads(run_json.read_text())
        run_data.pop("session_id", None)
        run_json.write_text(json.dumps(run_data))
        (Path(launched["run_dir"]) / "events.jsonl").write_text("")

        self._main(["resume", launched["run_dir"], "--message", "rework"])

        self.assertEqual(self.last_code, 64)

    def test_resume_default_timeout_uses_run_wall_clock_cap(self):
        launched = self.launch("commit", "--runtime", "codex", "--cap", "7")
        self.assertEqual(self.supervise(launched)["verdict"], "CLEAN")

        with mock.patch.object(worker.subprocess, "run") as run_mock:
            run_mock.return_value = subprocess.CompletedProcess([], 0, stdout="", stderr="")
            receipt = json.loads(self._main(["resume", launched["run_dir"], "--message", "rework"]))

        self.assertEqual(receipt["return_code"], 0)
        self.assertEqual(run_mock.call_args.kwargs["timeout"], 7)

    def test_resume_nonzero_exit_is_not_acknowledged(self):
        os.environ["FAKE_RESUME_EXIT"] = "7"
        launched = self.launch("commit", "--runtime", "codex")
        self.assertEqual(self.supervise(launched)["verdict"], "CLEAN")

        receipt = self.resume(launched, "please fix the review finding")

        self.assertEqual(self.last_code, worker.DEGRADED)
        self.assertEqual(receipt["return_code"], 7)
        self.assertFalse(receipt["acknowledged"])
        self.assertIn("exited 7", receipt["reason"])

    def test_resume_timeout_records_partial_events_and_degrades(self):
        os.environ["FAKE_RESUME_SLEEP"] = "5"
        launched = self.launch("commit", "--runtime", "codex")
        self.assertEqual(self.supervise(launched)["verdict"], "CLEAN")

        receipt = json.loads(self._main(["resume", launched["run_dir"], "--message", "rework", "--timeout", "0.1"]))

        self.assertEqual(self.last_code, worker.DEGRADED)
        self.assertTrue(receipt["timed_out"])
        self.assertIsNone(receipt["return_code"])
        self.assertIn("timed out", receipt["error"])
        self.assertIn("thread.started", Path(receipt["events_file"]).read_text())
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertTrue(run_data["resumes"][-1]["timed_out"])

    def test_resume_detached_starts_supervised_resume_process(self):
        os.environ["FAKE_RESUME_SLEEP"] = "5"
        launched = self.launch("commit", "--runtime", "codex", "--cap", "9", "--stall", "4")
        self.assertEqual(self.supervise(launched)["verdict"], "CLEAN")

        try:
            receipt = json.loads(self._main(["resume", launched["run_dir"], "--message", "rework", "--detach"]))

            self.assertEqual(self.last_code, worker.RUNNING)
            self.assertEqual(receipt["runtime"], "codex")
            self.assertTrue(receipt["detached"])
            self.assertIn("pid", receipt)
            self.assertEqual(receipt["timeout_seconds"], 9)
            self.assertEqual(receipt["stall_seconds"], 4)
            self.assertFalse(receipt["acknowledged"])
            self.assertIn("resume process started", receipt["reason"])
            self.assertTrue(Path(receipt["events_file"]).exists())
            run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
            self.assertEqual(run_data["pid"], receipt["pid"])
            self.assertEqual(run_data["started_epoch"], receipt["started_epoch"])
            self.assertEqual(run_data["resumes"][-1]["pid"], receipt["pid"])
        finally:
            self.stop({"run_dir": launched["run_dir"]})

    def test_supervise_detached_resume_uses_resume_event_stream_for_quiet_time(self):
        os.environ["FAKE_RESUME_SLEEP"] = "5"
        launched = self.launch("commit", "--runtime", "codex", "--stall", "1")
        self.assertEqual(self.supervise(launched)["verdict"], "CLEAN")
        old_events = Path(launched["run_dir"]) / "events.jsonl"
        old = time.time() - 10
        os.utime(old_events, (old, old))

        receipt = json.loads(self._main(["resume", launched["run_dir"], "--message", "rework", "--detach"]))
        try:
            result = self.supervise(launched, max_wait=0.2)
        finally:
            self.stop({"run_dir": launched["run_dir"]})

        self.assertEqual(self.last_code, worker.RUNNING)
        self.assertEqual(result["verdict"], "RUNNING")
        self.assertNotIn("stall", json.dumps(result["findings"]))
        self.assertEqual(receipt["events_file"], json.loads((Path(launched["run_dir"]) / "run.json").read_text())["active_events_file"])

    def test_terminal_delivery_without_worker_observation_is_unconfirmed(self):
        launched = self.launch("commit")
        first = self.supervise(launched)["candidate"]
        findings = Path(self.tmp.name) / "review-1.md"
        findings.write_text("Fix the missing assertion.\n")
        fake_bin = Path(self.tmp.name) / "bin"
        fake_bin.mkdir()
        fake_tmux = fake_bin / "tmux"
        fake_tmux.write_text("#!/bin/sh\nexit 0\n")
        fake_tmux.chmod(fake_tmux.stat().st_mode | stat.S_IXUSR)
        old_path = os.environ["PATH"]
        os.environ["PATH"] = f"{fake_bin}{os.pathsep}{old_path}"
        try:
            out = self._main([
                "deliver-findings",
                launched["run_dir"],
                "--findings-file",
                str(findings),
                "--transport",
                "terminal",
                "--terminal-target",
                "swallow",
                "--ack-timeout",
                "0.05",
            ])
        finally:
            os.environ["PATH"] = old_path

        self.assertEqual(self.last_code, worker.DEGRADED)
        delivery = json.loads(out)
        self.assertEqual(delivery["candidate"], first["commit"])
        self.assertTrue(delivery["counts_toward_cap"])
        self.assertEqual(delivery["round"], 1)
        self.assertEqual(delivery["receipt"]["transport"], "terminal")
        self.assertTrue(delivery["receipt"]["sent"])
        self.assertFalse(delivery["receipt"]["acknowledged"])
        self.assertIn("no worker acknowledgement", delivery["receipt"]["reason"])
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertTrue(run_data["rework_deliveries"][-1]["counts_toward_cap"])
        self.assertNotIn("review", run_data["candidates"][0])
        with (Path(launched["run_dir"]) / "events.jsonl").open("a") as handle:
            handle.write(json.dumps({"type": "item.completed", "item": {"type": "command_execution", "command": "read review findings"}}) + "\n")
        self.status(launched, update_run_record=True)
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertFalse(run_data["rework_deliveries"][-1]["receipt"]["acknowledged"])
        self.assertNotIn("review", run_data["candidates"][0])
        digest = worker.digest_file(findings)
        with (Path(launched["run_dir"]) / "events.jsonl").open("a") as handle:
            handle.write(
                json.dumps({
                    "type": "item.completed",
                    "item": {
                        "type": "command_execution",
                        "command": f"cat {findings.resolve()} # {digest}",
                    },
                }) + "\n"
            )
        self.status(launched, update_run_record=True)
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertTrue(run_data["rework_deliveries"][-1]["counts_toward_cap"])
        self.assertTrue(run_data["rework_deliveries"][-1]["receipt"]["acknowledged"])
        self.assertEqual(run_data["rework_deliveries"][-1]["receipt"]["acknowledgement"]["kind"], "event")
        self.assertEqual(run_data["candidates"][0]["review"]["delivery_id"], delivery["id"])

    def test_status_does_not_acknowledge_pending_terminal_delivery_from_lead_pane_echo(self):
        launched = self.launch("commit")
        first = self.supervise(launched)["candidate"]
        findings = Path(self.tmp.name) / "review-1.md"
        findings.write_text("Fix the missing assertion.\n")
        pane = Path(self.tmp.name) / "pane.txt"
        pane.write_text("")
        fake_bin = Path(self.tmp.name) / "bin"
        fake_bin.mkdir()
        fake_tmux = fake_bin / "tmux"
        fake_tmux.write_text(
            "#!/bin/sh\n"
            "if [ \"$1\" = capture-pane ]; then\n"
            "  cat \"$CORENIUS_FAKE_PANE\"\n"
            "fi\n"
            "exit 0\n"
        )
        fake_tmux.chmod(fake_tmux.stat().st_mode | stat.S_IXUSR)
        old_path = os.environ["PATH"]
        os.environ["PATH"] = f"{fake_bin}{os.pathsep}{old_path}"
        os.environ["CORENIUS_FAKE_PANE"] = str(pane)
        try:
            out = self._main([
                "deliver-findings",
                launched["run_dir"],
                "--findings-file",
                str(findings),
                "--transport",
                "terminal",
                "--terminal-target",
                "plain-pane",
                "--ack-timeout",
                "0.05",
            ])
            self.assertEqual(self.last_code, worker.DEGRADED)
            delivery = json.loads(out)
            pane.write_text(f"Read {findings.resolve()} now.\n")
            self.status(launched, update_run_record=True)
        finally:
            os.environ["PATH"] = old_path
            os.environ.pop("CORENIUS_FAKE_PANE", None)

        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertEqual(delivery["candidate"], first["commit"])
        self.assertFalse(run_data["rework_deliveries"][-1]["receipt"]["acknowledged"])
        self.assertNotIn("review", run_data["candidates"][0])

    def test_pane_acknowledgement_tolerates_wrapped_findings_path(self):
        findings = Path(self.tmp.name) / "very" / "long" / "review-1.md"
        findings.parent.mkdir(parents=True)
        findings.write_text("Fix it.\n")
        wrapped = str(findings.resolve()).replace("/long/", "/long/\n")

        with mock.patch.object(worker, "tmux_pane_text", return_value=f"Current activity: received {wrapped}\n"):
            acknowledgement = worker.pane_acknowledgement("pane", "", findings)

        self.assertEqual(acknowledgement, {"kind": "pane-activity", "target": "pane"})

    def test_stale_base_successor_consumes_older_review_delivery(self):
        run = {
            "candidates": [
                {"candidate": {"commit": "a" * 40}},
                {"candidate": {"commit": "b" * 40}, "source_delivery_id": "stale-base-2"},
            ],
            "rework_deliveries": [
                {"id": "review-1", "kind": "review", "receipt": {"acknowledged": True}},
                {"id": "stale-base-2", "kind": "stale-base", "receipt": {"acknowledged": True}},
            ],
        }

        self.assertIsNone(worker.pending_delivery(run))

    def test_deliver_findings_records_digest_candidate_round_and_terminal_receipt(self):
        launched = self.launch("commit")
        first = self.supervise(launched)["candidate"]
        findings = Path(self.tmp.name) / "review-1.md"
        findings.write_text("Fix the missing assertion.\n")
        terminal = Path(self.tmp.name) / "terminal-input.txt"
        expected_digest = worker.digest_file(findings)
        thread = self.append_event_after(
            launched,
            {
                "type": "item.completed",
                "item": {
                    "type": "command_execution",
                    "command": f"cat {findings.resolve()} # {expected_digest}",
                },
            },
        )

        out = self._main([
            "deliver-findings",
            launched["run_dir"],
            "--findings-file",
            str(findings),
            "--transport",
            "terminal",
            "--terminal-input-file",
            str(terminal),
            "--ack-timeout",
            "1",
        ])
        thread.join()

        self.assertEqual(self.last_code, worker.CLEAN)
        delivery = json.loads(out)
        self.assertEqual(delivery["kind"], "review")
        self.assertEqual(delivery["round"], 1)
        self.assertTrue(delivery["counts_toward_cap"])
        self.assertEqual(delivery["candidate"], first["commit"])
        self.assertEqual(delivery["findings_sha256"], expected_digest)
        self.assertEqual(delivery["receipt"]["transport"], "terminal")
        self.assertTrue(delivery["receipt"]["sent"])
        self.assertEqual(delivery["receipt"]["acknowledged"], True)
        self.assertEqual(delivery["receipt"]["acknowledgement"]["kind"], "event")
        self.assertEqual(delivery["receipt"]["acknowledgement"]["event_type"], "item.completed")
        typed = terminal.read_text()
        self.assertIn(str(findings.resolve()), typed)
        self.assertIn(expected_digest, typed)
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertEqual(run_data["rework_deliveries"][-1]["id"], delivery["id"])
        self.assertEqual(run_data["candidates"][0]["candidate"], first)
        self.assertEqual(run_data["candidates"][0]["review"]["delivery_id"], delivery["id"])

    def test_new_marker_after_delivered_findings_is_recorded_as_successor_candidate(self):
        marker = "ticket 01: ready for review"
        os.environ["FAKE_MARKER"] = marker
        launched = self.launch("marker_alive", "--handoff-mode", "marker", "--handoff-marker", marker)
        try:
            first = self.supervise(launched, max_wait=5)["candidate"]
            findings = Path(self.tmp.name) / "review-1.md"
            findings.write_text("Fix result.txt.\n")
            thread = self.append_event_after(launched, {
                "type": "item.completed",
                "item": {
                    "type": "command_execution",
                    "command": f"read review findings {findings.resolve()}",
                },
            })
            self._main([
                "deliver-findings",
                launched["run_dir"],
                "--findings-file",
                str(findings),
                "--terminal-input-file",
                str(Path(self.tmp.name) / "terminal.txt"),
                "--ack-timeout",
                "1",
            ])
            thread.join()
            waiting = self.status(launched, update_run_record=True)
            self.assertFalse(waiting["ready_for_review"], waiting)
            self.assertIn("awaiting-rework", waiting["not_ready"])

            worktree = Path(launched["worktree"])
            (worktree / "out.md").write_text("fixed\n")
            run(worktree, "add", "out.md")
            run(worktree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "--allow-empty", "-qm", "fix")
            reintegration = self.status(launched, update_run_record=True)
            self.assertFalse(reintegration["ready_for_review"], reintegration)
            self.assertIn("fixes-committed-waiting-for-re-integration", reintegration["not_ready"])
            run(worktree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "--allow-empty", "-qm", marker)
            status = self.status(launched, update_run_record=True)

            self.assertTrue(status["ready_for_review"], status)
            self.assertNotIn("candidate-moved", json.dumps(status["findings"]))
            self.assertNotEqual(status["candidate"]["commit"], first["commit"])
            run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
            self.assertEqual([entry["round"] for entry in run_data["candidates"]], [0, 1])
            self.assertEqual(run_data["candidates"][0]["candidate"]["commit"], first["commit"])
            self.assertEqual(run_data["candidates"][1]["candidate"], status["candidate"])
            self.assertEqual(run_data["candidates"][1]["source_delivery_id"], run_data["rework_deliveries"][-1]["id"])
        finally:
            self.stop(launched)

    def test_launch_help_lists_runtime_observation_limits(self):
        proc = subprocess.run(
            [sys.executable, str(Path(worker.__file__)), "launch", "--help"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("--runtime", proc.stdout)
        self.assertIn("traecli observed:", proc.stdout)
        self.assertIn("codex observed:", proc.stdout)
        self.assertIn("codex not observed:", proc.stdout)
        self.assertIn("traecli access profile:", proc.stdout)
        self.assertIn("codex access profile:", proc.stdout)
        self.assertIn("enforced:", proc.stdout)
        self.assertIn("declared:", proc.stdout)
        self.assertIn("detected:", proc.stdout)

    def test_supervise_help_lists_workspace_change_attribution_limits(self):
        proc = subprocess.run(
            [sys.executable, str(Path(worker.__file__)), "supervise", "--help"],
            capture_output=True,
            text=True,
        )

        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("unaccounted workspace changes", proc.stdout)
        self.assertIn("cannot identify the writer process", proc.stdout)

    def test_status_help_discloses_run_record_updates(self):
        proc = subprocess.run(
            [sys.executable, str(Path(worker.__file__)), "status", "--help"],
            capture_output=True,
            text=True,
        )

        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("--update-run-record", proc.stdout)
        self.assertIn("write observed Candidate", proc.stdout)

    def test_status_default_writes_no_run_record(self):
        launched = self.launch("commit")
        run_path = Path(launched["run_dir"]) / "run.json"
        self.wait_for_exit(launched)
        before = run_path.read_text()

        status = self.status(launched)

        self.assertTrue(status["ready_for_review"])
        self.assertEqual(run_path.read_text(), before)

    def test_status_update_run_record_persists_candidate_when_explicit(self):
        launched = self.launch("commit")
        self.wait_for_exit(launched)
        out = self._main(["status", launched["run_dir"], "--update-run-record"])
        status = json.loads(out)
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())

        self.assertTrue(status["ready_for_review"])
        self.assertEqual(run_data["candidate"], status["candidate"])

    def test_launch_validates_options_before_creating_run_or_clone(self):
        out = self._main([
            "launch",
            "--task", str(self.task),
            "--name", "bad-marker",
            "--allow", "out.md",
            "--repo", str(self.repo),
            "--handoff-mode", "marker",
        ])

        self.assertEqual(self.last_code, 64)
        self.assertEqual(out, "")
        self.assertFalse((Path(os.environ["CORENIUS_RUNS"]) / "workers").exists())

    def test_launch_validates_base_before_creating_run_or_clone(self):
        out = self._main([
            "launch",
            "--task", str(self.task),
            "--name", "bad-base",
            "--allow", "out.md",
            "--repo", str(self.repo),
            "--base", "missing-branch",
        ])

        self.assertEqual(self.last_code, 64)
        self.assertEqual(out, "")
        self.assertFalse((Path(os.environ["CORENIUS_RUNS"]) / "workers").exists())
        self.assertFalse((self.repo.parent / f"{self.repo.name}-worker-bad-base").exists())

    def test_launch_validates_remaining_options_before_creating_run_or_clone(self):
        cases = [
            ("bad-allow", ["--allow", "../escape"]),
            ("bad-protect", ["--protect", str(self.root / "missing-protect")]),
            ("bad-external-add", ["--external-add-path", str(self.repo / "scratch")]),
            ("bad-timeout", ["--command-timeout", "0"]),
        ]
        for name, extra in cases:
            with self.subTest(name=name):
                out = self._main([
                    "launch",
                    "--task", str(self.task),
                    "--name", name,
                    "--allow", "out.md",
                    "--repo", str(self.repo),
                    *extra,
                ])

                self.assertEqual(self.last_code, 64)
                self.assertEqual(out, "")
                self.assertFalse((Path(os.environ["CORENIUS_RUNS"]) / "workers").exists())
                self.assertFalse((self.repo.parent / f"{self.repo.name}-worker-{name}").exists())

    def test_launch_base_branch_resolves_in_requested_repo(self):
        integration = self.root / "integration"
        run(self.repo, "worktree", "add", "-q", "-b", "effort/demo", str(integration), "main")
        (integration / "integration.txt").write_text("tip\n")
        run(integration, "add", "integration.txt")
        run(integration, "commit", "-qm", "integration tip")

        launched = self._main([
            "launch",
            "--task", str(self.task),
            "--name", "from-integration",
            "--allow", "out.md",
            "--repo", str(integration),
            "--base", "effort/demo",
        ])
        run_data = json.loads((Path(json.loads(launched)["run_dir"]) / "run.json").read_text())

        self.assertEqual(run_data["base_sha"], run(integration, "rev-parse", "effort/demo").strip())

    def test_default_access_profile_records_network_on_and_allows_downloads(self):
        launched = self.launch("network")
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertEqual(run_data["access_profile"]["network"], "on")
        self.assertEqual(run_data["access_profile"]["command_timeout_seconds"], 120)
        support = run_data["access_profile_support"]
        self.assertIn("traecli", support)
        self.assertIn("codex", support)
        self.assertIn("network", support["traecli"]["declared"])
        self.assertIn("command_timeout_seconds", support["traecli"]["enforced"])
        self.assertIn("network-command", support["codex"]["detected"])
        self.assertNotIn(r"\b(?:curl|wget)\b", run_data["forbidden"])
        prompt = (Path(launched["run_dir"]) / "prompt.md").read_text()
        self.assertIn("Network access is available.", prompt)
        self.assertNotIn("No network access", prompt)
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertNotIn("network-command", json.dumps(verdict["findings"]))
        self.assertNotIn("forbidden-command", json.dumps(verdict["findings"]))

    def test_network_off_records_prompt_and_flags_download_command(self):
        launched = self.launch("network", "--network", "off")
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertEqual(run_data["access_profile"]["network"], "off")
        prompt = (Path(launched["run_dir"]) / "prompt.md").read_text()
        self.assertIn("Network access is off for this launch.", prompt)
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "VIOLATED", verdict)
        self.assertIn("network-command", json.dumps(verdict["findings"]))

    def test_command_timeout_is_a_launch_option_recorded_and_passed_to_worker(self):
        launched = self.launch("timeout_probe", "--command-timeout", "125")
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertEqual(run_data["access_profile"]["command_timeout_seconds"], 125)
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertEqual((Path(launched["worktree"]) / "out.md").read_text(), "125s\n")

    def test_launch_sets_run_owned_temp_environment_and_supervise_removes_it(self):
        forbidden = self.root / "forbidden-host-tmp"
        forbidden.mkdir()
        before = self.tree_listing(forbidden)
        with mock.patch.dict(os.environ, {"CORENIUS_FORBIDDEN_TMP": str(forbidden)}, clear=False):
            for name in ("TMPDIR", "TMUX_TMPDIR", "TMPPREFIX"):
                os.environ.pop(name, None)
            launched = self.launch("temp_probe")

        self.wait_for_exit(launched)
        run_path = Path(launched["run_dir"]) / "run.json"
        run_data = json.loads(run_path.read_text())
        temp = run_data["temp"]
        out_path = Path(launched["worktree"]) / "out.md"
        self.assertTrue(out_path.exists(), (Path(launched["run_dir"]) / "stderr.log").read_text())
        observed = json.loads(out_path.read_text())

        self.assertTrue(Path(temp["tmpdir"]).is_relative_to(Path(launched["run_dir"])))
        self.assertEqual(observed["TMPDIR"], temp["tmpdir"])
        self.assertEqual(observed["TMUX_TMPDIR"], temp["tmux_tmpdir"])
        self.assertEqual(observed["TMPPREFIX"], temp["tmpprefix"])
        self.assertTrue(Path(observed["mkdtemp"]).is_relative_to(Path(temp["tmpdir"])))
        self.assertTrue(Path(observed["here_doc"]).is_relative_to(Path(temp["tmpdir"])))
        self.assertIn(temp["tmpdir"], (Path(launched["run_dir"]) / "prompt.md").read_text())
        self.assertEqual(self.tree_listing(forbidden), before)

        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertFalse(Path(temp["tmpdir"]).exists())

    def test_workspace_write_runtime_gets_temp_directory_as_writable_root(self):
        log = self.root / "launch-log.jsonl"
        os.environ["FAKE_LAUNCH_LOG"] = str(log)

        launched = self.launch("clean", "--runtime", "codex")
        self.wait_for_exit(launched)

        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        argv = json.loads(log.read_text().splitlines()[-1])["argv"]
        self.assertIn(run_data["temp"]["tmpdir"], argv)

    def test_external_add_paths_and_restricted_capabilities_are_recorded_in_prompt(self):
        external = Path(self.tmp.name) / "outside-adds"
        launched = self.launch("clean", "--external-add-path", str(external))
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        profile = run_data["access_profile"]
        self.assertEqual(profile["external_add_paths"], [str(external.resolve())])
        self.assertFalse(profile["container_builds"])
        self.assertFalse(profile["shared_cache_writes"])
        prompt = (Path(launched["run_dir"]) / "prompt.md").read_text()
        self.assertIn(str(external.resolve()), prompt)
        self.assertIn("Container builds are not allowed.", prompt)
        self.assertIn("Writes to shared caches are not allowed.", prompt)
        self.assertEqual(self.supervise(launched)["verdict"], "CLEAN")

        declared = self.launch("clean", "--allow-container-builds", "--allow-shared-cache-writes")
        declared_run = json.loads((Path(declared["run_dir"]) / "run.json").read_text())
        self.assertTrue(declared_run["access_profile"]["container_builds"])
        self.assertTrue(declared_run["access_profile"]["shared_cache_writes"])
        declared_prompt = (Path(declared["run_dir"]) / "prompt.md").read_text()
        self.assertIn("Container builds are allowed.", declared_prompt)
        self.assertIn("Writes to shared caches are allowed.", declared_prompt)
        self.assertEqual(self.supervise(declared)["verdict"], "CLEAN")

    def test_external_protected_path_is_validated_before_worktree_creation(self):
        root = Path(self.tmp.name)
        missing = root / "missing-protected"
        in_repo = self.repo / "README"
        run_root = Path(os.environ["CORENIUS_RUNS"])
        run_root.mkdir()
        branches = run(self.repo, "for-each-ref", "--format=%(refname:short)", "refs/heads").splitlines()

        for bad in (missing, "relative-protected", in_repo, root, run_root):
            self.name += 1
            self._main(
                ["launch", "--task", str(self.task), "--name", f"bad{self.name}", "--allow", "out.md",
                 "--repo", str(self.repo), "--external-protect-path", str(bad)]
            )
            self.assertEqual(self.last_code, 64)

        self.assertEqual(run(self.repo, "for-each-ref", "--format=%(refname:short)", "refs/heads").splitlines(), branches)
        self.assertEqual(list(root.glob("corenius-worker-bad*")), [])

    def test_external_protected_baseline_precedes_worker_start(self):
        protected = Path(self.tmp.name) / "shared"
        protected.mkdir()
        (protected / "state.txt").write_text("before\n")

        launched = self.launch("slow_commit", "--external-protect-path", str(protected), "--stall", "5")
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        baseline = Path(run_data["external_state"]["baseline_file"])

        self.assertTrue(baseline.is_file())
        self.assertLess(baseline.stat().st_mtime, run_data["started_epoch"])
        self.assertEqual(run_data["external_state"]["protected_paths"], [str(protected.resolve())])
        self.assertEqual(run_data["external_state"]["baseline_entry_count"], 2)
        self.assertIsInstance(run_data["external_state"]["baseline_scan_seconds"], float)
        self.assertEqual(self.supervise(launched)["verdict"], "CLEAN")
        run_after = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertEqual(run_after["external_state"]["last_scan"]["entry_count"], 2)
        self.assertIsInstance(run_after["external_state"]["last_scan"]["scan_seconds"], float)

    def test_external_protected_changes_report_counts_and_ignore_declared_add_path(self):
        protected = Path(self.tmp.name) / "shared"
        add_path = protected / "allowed-adds"
        add_path.mkdir(parents=True)
        changed = protected / "changed.txt"
        removed = protected / "removed.txt"
        changed.write_text("before\n")
        removed.write_text("remove me\n")

        launched = self.launch(
            "slow_commit",
            "--external-protect-path", str(protected),
            "--external-add-path", str(add_path),
            "--stall", "5",
        )
        changed.write_text("after\n")
        removed.unlink()
        (protected / "new.txt").write_text("new\n")
        (add_path / "ignored.txt").write_text("allowed\n")

        verdict = self.supervise(launched)

        self.assertEqual(verdict["verdict"], "VIOLATED", verdict)
        details = {f["kind"]: f["detail"] for f in verdict["findings"] if f["kind"].startswith("external-state-")}
        self.assertIn("external-state-modified", details)
        self.assertIn("external-state-removed", details)
        self.assertIn("external-state-added", details)
        self.assertIn("modified=1 removed=0 added=0", details["external-state-modified"])
        self.assertIn("modified=0 removed=1 added=0", details["external-state-removed"])
        self.assertIn("modified=0 removed=0 added=1", details["external-state-added"])
        self.assertIn("changed during the run", " ".join(details.values()))
        self.assertNotIn("worker", " ".join(details.values()).lower())
        self.assertNotIn("ignored.txt", json.dumps(verdict["findings"]))

    def test_external_protected_change_labels_include_disambiguating_root(self):
        root_a = Path(self.tmp.name) / "root-a"
        root_b = Path(self.tmp.name) / "root-b"
        labels = worker.external_change_labels(
            {"external_state": {"protected_paths": [str(root_a), str(root_b)]}},
            [str(root_a / "state.txt"), str(root_b / "state.txt")],
        )

        self.assertEqual(labels, ["external[1]:root-a/state.txt", "external[2]:root-b/state.txt"])

    def test_external_protected_final_digest_catches_same_size_same_mtime_change(self):
        protected = Path(self.tmp.name) / "shared"
        protected.mkdir()
        target = protected / "same.txt"
        target.write_text("aaaa\n")
        before = target.stat()

        launched = self.launch("slow_commit", "--external-protect-path", str(protected), "--stall", "5")
        target.write_text("bbbb\n")
        os.utime(target, ns=(before.st_atime_ns, before.st_mtime_ns))

        verdict = self.supervise(launched)

        self.assertEqual(verdict["verdict"], "VIOLATED", verdict)
        self.assertIn("external-state-modified", json.dumps(verdict["findings"]))
        self.assertIn("same.txt", json.dumps(verdict["findings"]))

    def test_external_protected_symlink_records_target_and_is_not_followed(self):
        protected = Path(self.tmp.name) / "shared"
        protected.mkdir()
        outside = Path(self.tmp.name) / "outside"
        outside.mkdir()
        (outside / "secret.txt").write_text("do not walk\n")
        link = protected / "link"
        link.symlink_to(outside)

        launched = self.launch("clean", "--external-protect-path", str(protected))
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        baseline = json.loads(Path(run_data["external_state"]["baseline_file"]).read_text())
        entries = {entry["relative"]: entry for entry in baseline["entries"]}

        self.assertEqual(entries["link"]["kind"], "symlink")
        self.assertEqual(entries["link"]["target"], str(outside))
        self.assertNotIn("link/secret.txt", entries)
        self.assertEqual(self.supervise(launched)["verdict"], "CLEAN")

    def test_container_builds_and_shared_cache_writes_follow_the_profile(self):
        container = self.supervise(self.launch("container_build"))
        self.assertEqual(container["verdict"], "VIOLATED", container)
        self.assertIn("container-build", json.dumps(container["findings"]))

        cache = self.supervise(self.launch("shared_cache"))
        self.assertEqual(cache["verdict"], "VIOLATED", cache)
        self.assertIn("shared-cache-write", json.dumps(cache["findings"]))

        container_allowed = self.supervise(self.launch("container_build", "--allow-container-builds"))
        self.assertEqual(container_allowed["verdict"], "CLEAN", container_allowed)
        self.assertNotIn("container-build", json.dumps(container_allowed["findings"]))

        cache_allowed = self.supervise(self.launch("shared_cache", "--allow-shared-cache-writes"))
        self.assertEqual(cache_allowed["verdict"], "CLEAN", cache_allowed)
        self.assertNotIn("shared-cache-write", json.dumps(cache_allowed["findings"]))

    def test_unexpected_file_is_a_violation(self):
        verdict = self.supervise(self.launch("extra"))
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertEqual(self.last_code, worker.VIOLATED)
        self.assertIn("stray.txt", json.dumps(verdict["findings"]))

    def test_allowed_dirty_file_not_named_by_worker_events_is_a_note_while_running(self):
        launched = self.launch("stall")
        try:
            target = Path(launched["worktree"]) / "out.md"
            target.write_text("changed outside worker event stream\n")

            status = self.status(launched)

            self.assertIn(
                {
                    "severity": "NOTE",
                    "kind": "unaccounted-worktree-change",
                    "detail": "allowed path changed without a recorded worker command or file-change event naming it: out.md",
                },
                status["findings"],
            )
            self.assertNotIn(
                {
                    "severity": "VIOLATION",
                    "kind": "unaccounted-worktree-change",
                    "detail": "allowed path changed without a recorded worker command or file-change event naming it: out.md",
                },
                status["findings"],
            )
        finally:
            self.stop(launched)

    def test_allowed_dirty_file_not_named_by_worker_events_is_a_violation_after_candidate(self):
        marker = "ticket 01: ready for review"
        os.environ["FAKE_MARKER"] = marker
        launched = self.launch("marker_alive", "--handoff-mode", "marker", "--handoff-marker", marker)
        try:
            target = Path(launched["worktree"]) / "out.md"
            deadline = time.time() + 5
            while not target.exists() and time.time() < deadline:
                time.sleep(0.1)
            self.assertTrue(target.exists())
            clean = self.supervise(launched, max_wait=5)
            self.assertTrue(clean["ready_for_review"], clean)
            with (Path(launched["run_dir"]) / "events.jsonl").open("a") as handle:
                handle.write(json.dumps({
                    "type": "item.completed",
                    "item": {"id": "late-edit", "type": "command_execution", "command": "write out.md", "status": "completed"},
                }) + "\n")
            target.write_text("changed outside worker event stream\n")

            status = self.status(launched)

            self.assertIn(
                {
                    "severity": "VIOLATION",
                    "kind": "unaccounted-worktree-change",
                    "detail": "ready candidate is followed by an allowed dirty path not accounted for by the event record: out.md",
                },
                status["findings"],
            )
        finally:
            self.stop(launched)

    def test_dirty_tree_is_not_ready_for_review(self):
        verdict = self.supervise(self.launch("dirty"))
        self.assertEqual(verdict["verdict"], "DEGRADED", verdict)
        self.assertFalse(verdict["ready_for_review"])
        self.assertIsNone(verdict["candidate"])
        self.assertIn("dirty-tree", json.dumps(verdict["not_ready"]))

    def test_exiting_without_commit_is_not_ready_for_review(self):
        verdict = self.supervise(self.launch("no_commit"))
        self.assertEqual(verdict["verdict"], "DEGRADED", verdict)
        self.assertFalse(verdict["ready_for_review"])
        self.assertIsNone(verdict["candidate"])
        self.assertIn("no-commit-after-base", json.dumps(verdict["not_ready"]))

    def test_commit_on_own_branch_is_ready_not_a_violation(self):
        verdict = self.supervise(self.launch("commit"))
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertTrue(verdict["ready_for_review"])
        self.assertNotIn("worker-committed", json.dumps(verdict["findings"]))

    def test_shared_repo_peer_worker_branch_import_is_not_a_violation(self):
        launched = self.launch("commit")
        run_path = Path(launched["run_dir"]) / "run.json"
        run_data = json.loads(run_path.read_text())
        peer_branch = "worker/demo-peer"
        run_data["peer_worker_branches"] = [peer_branch]
        run_path.write_text(json.dumps(run_data))

        run(self.repo, "branch", peer_branch, "HEAD")
        verdict = self.supervise(launched)

        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertTrue(verdict["ready_for_review"])
        self.assertNotIn("shared-branch-moved", json.dumps(verdict["findings"]))

    def test_shared_repository_branch_movement_by_other_agents_is_non_blocking_note(self):
        run(self.repo, "branch", "effort/demo", "main")
        launched = self.launch("slow_commit", "--stall", "10")
        unrelated = self.root / "unrelated-agent"
        new_branch = self.root / "new-agent"
        integration = self.root / "integration"
        run(self.repo, "worktree", "add", "-q", str(unrelated), "side")
        run(self.repo, "worktree", "add", "-q", "-b", "agent/new-branch", str(new_branch), "main")
        run(self.repo, "worktree", "add", "-q", str(integration), "effort/demo")
        for checkout, name in (
            (unrelated, "side.txt"),
            (new_branch, "new.txt"),
            (integration, "integration.txt"),
        ):
            (checkout / name).write_text(f"{name}\n")
            run(checkout, "add", name)
            run(checkout, "commit", "-qm", f"move {name}")

        verdict = self.supervise(launched)

        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertTrue(verdict["ready_for_review"])
        self.assertNotIn("boundary-violation", verdict["not_ready"])
        branch_findings = [f for f in verdict["findings"] if f["kind"] == "shared-branch-moved"]
        self.assertTrue(branch_findings)
        self.assertTrue(all(f["severity"] == "NOTE" for f in branch_findings))
        self.assertNotIn("worker/", json.dumps(branch_findings))

    def test_moving_a_foreign_branch_is_a_violation(self):
        side_before = run(self.repo, "rev-parse", "side").strip()
        verdict = self.supervise(self.launch("move_main"))
        self.assertEqual(verdict["verdict"], "VIOLATED", verdict)
        self.assertFalse(verdict["ready_for_review"])
        self.assertIn("foreign-branch-moved", json.dumps(verdict["findings"]))
        self.assertEqual(run(self.repo, "rev-parse", "side").strip(), side_before)

    def test_branch_that_no_longer_contains_base_is_a_violation(self):
        verdict = self.supervise(self.launch("rewrite_base"))
        self.assertEqual(verdict["verdict"], "VIOLATED", verdict)
        self.assertFalse(verdict["ready_for_review"])
        self.assertIn("base-not-ancestor", json.dumps(verdict["findings"]))

    def test_marker_handoff_reports_ready_before_process_exit_and_pins_candidate(self):
        marker = "ticket 01: ready for review"
        os.environ["FAKE_MARKER"] = marker
        launched = self.launch("marker_alive", "--handoff-mode", "marker", "--handoff-marker", marker)
        try:
            verdict = self.supervise(launched, max_wait=5)
            self.assertEqual(verdict["verdict"], "CLEAN", verdict)
            self.assertTrue(verdict["ready_for_review"])
            self.assertTrue(self.status(launched)["alive"])
            run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
            self.assertEqual(run_data["candidate"], verdict["candidate"])
            self.assertEqual(verdict["candidate"]["commit"], run(self.repo, "rev-parse", launched["branch"]).strip())
        finally:
            self.stop(launched)

    def test_running_limit_notices_are_written_to_status_log(self):
        launched = self.launch("stall")
        events = Path(launched["run_dir"]) / "events.jsonl"
        deadline = time.time() + 5
        while events.stat().st_size == 0 and time.time() < deadline:
            time.sleep(0.1)
        self.assertGreater(events.stat().st_size, 0)
        run_json = Path(launched["run_dir"]) / "run.json"
        run_data = json.loads(run_json.read_text())
        run_data["started_epoch"] = time.time() - 5
        run_data["cap_seconds"] = 1
        run_json.write_text(json.dumps(run_data, indent=2) + "\n")
        old = time.time() - 5
        os.utime(events, (old, old))
        try:
            result = self.supervise(launched, max_wait=0.4)
        finally:
            self.stop(launched)

        self.assertEqual(result["verdict"], "RUNNING")
        rows = [json.loads(line) for line in (Path(launched["run_dir"]) / "status.jsonl").read_text().splitlines()]
        notices = [notice["kind"] for row in rows for notice in row.get("notices", [])]
        self.assertIn("wall-clock-limit", notices)
        self.assertIn("quiet-time", notices)

    def test_wall_clock_limit_notice_is_written_when_timeout_kills_at_cap(self):
        launched = self.launch("stall", "--cap", "1")

        result = self.supervise(launched, max_wait=3)

        self.assertEqual(result["verdict"], "DEGRADED")
        rows = [json.loads(line) for line in (Path(launched["run_dir"]) / "status.jsonl").read_text().splitlines()]
        notices = [notice["kind"] for row in rows for notice in row.get("notices", [])]
        self.assertIn("wall-clock-limit", notices)

    def test_marker_handoff_with_dirty_tree_is_not_ready(self):
        marker = "ticket 01: ready for review"
        os.environ["FAKE_MARKER"] = marker
        verdict = self.supervise(self.launch("marker_dirty", "--handoff-mode", "marker", "--handoff-marker", marker))
        self.assertEqual(verdict["verdict"], "DEGRADED", verdict)
        self.assertFalse(verdict["ready_for_review"])
        self.assertIsNone(verdict["candidate"])
        self.assertIn("dirty-tree", verdict["not_ready"])

    def test_candidate_is_pinned_and_later_tip_movement_is_reported(self):
        marker = "ticket 01: ready for review"
        os.environ["FAKE_MARKER"] = marker
        launched = self.launch("marker_move", "--handoff-mode", "marker", "--handoff-marker", marker)
        try:
            verdict = self.supervise(launched, max_wait=5)
            first = verdict["candidate"]["commit"]
            deadline = time.time() + 5
            while run(Path(launched["worktree"]), "rev-parse", "HEAD").strip() == first and time.time() < deadline:
                time.sleep(0.1)
            moved = run(Path(launched["worktree"]), "rev-parse", "HEAD").strip()
            self.assertNotEqual(first, moved)
            status = self.status(launched)
            self.assertTrue(status["ready_for_review"])
            self.assertEqual(status["candidate"]["commit"], first)
            self.assertIn("candidate-moved", json.dumps(status["findings"]))
            self.assertIn(first, json.dumps(status["findings"]))
            self.assertIn(moved, json.dumps(status["findings"]))
            later = self.supervise(launched, max_wait=5)
            self.assertEqual(later["candidate"]["commit"], first)
            self.assertIn("candidate-moved", json.dumps(later["findings"]))
        finally:
            self.stop(launched)

    def test_forbidden_command_is_a_violation(self):
        verdict = self.supervise(self.launch("forbidden", "--network", "off"))
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertIn("curl", json.dumps(verdict["findings"]))

    def test_rm_inside_worktree_is_a_note_not_a_violation(self):
        verdict = self.supervise(self.launch("rm_scratch"))
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        rm = next(finding for finding in verdict["findings"] if finding["kind"] == "rm-recursive")
        self.assertEqual(rm["severity"], "NOTE")

    def test_rm_of_home_or_parent_is_a_violation(self):
        for mode in ("rm_home", "rm_parent"):
            verdict = self.supervise(self.launch(mode))
            self.assertEqual(verdict["verdict"], "VIOLATED", (mode, verdict))
            self.assertIn("rm-outside-worktree", json.dumps(verdict["findings"]))

    def test_rm_of_tracked_file_is_caught_by_the_diff(self):
        verdict = self.supervise(self.launch("rm_tracked"))
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertIn("unexpected-change", json.dumps(verdict["findings"]))

    def test_old_run_with_recorded_rm_regex_is_rejudged(self):
        launched = self.launch("rm_scratch")
        run_json = Path(launched["run_dir"]) / "run.json"
        run = json.loads(run_json.read_text())
        run["forbidden"].append(worker.LEGACY_RM)
        run_json.write_text(json.dumps(run))
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)

    def test_run_recorded_before_access_profiles_still_supervises(self):
        launched = self.launch("clean")
        deadline = time.time() + 5
        exit_code = Path(launched["run_dir"]) / "exit_code"
        while not exit_code.exists() and time.time() < deadline:
            time.sleep(0.1)
        run_json = Path(launched["run_dir"]) / "run.json"
        run_data = json.loads(run_json.read_text())
        run_data.pop("access_profile", None)
        run_json.write_text(json.dumps(run_data))
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)

    def test_rerun_keeps_the_previous_verdict(self):
        launched = self.launch("clean")
        self.supervise(launched)
        self.supervise(launched)
        self.assertTrue((Path(launched["run_dir"]) / "verdict.prev.json").exists())

    def test_read_only_git_is_allowed(self):
        verdict = self.supervise(self.launch("clean"))
        self.assertNotIn("forbidden-command", json.dumps(verdict["findings"]))

    def test_protected_repo_change_is_a_violation(self):
        other = Path(self.tmp.name) / "other"
        other.mkdir()
        run(other, "init", "-q", "-b", "main")
        run(other, "config", "user.email", "a@b")
        run(other, "config", "user.name", "a")
        (other / "f").write_text("1\n")
        run(other, "add", "f")
        run(other, "commit", "-qm", "1")
        launched = self.launch("clean", "--protect", str(other))
        (other / "f").write_text("2\n")
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertIn("protected-dirty-changed", json.dumps(verdict["findings"]))

    def make_other(self):
        other = Path(self.tmp.name) / "other"
        other.mkdir()
        run(other, "init", "-q", "-b", "main")
        run(other, "config", "user.email", "a@b")
        run(other, "config", "user.name", "a")
        (other / "f").write_text("1\n")
        run(other, "add", "f")
        run(other, "commit", "-qm", "1")
        return other

    def test_protected_head_fast_forward_is_a_note(self):
        other = self.make_other()
        launched = self.launch("clean", "--protect", str(other))
        (other / "g").write_text("2\n")
        run(other, "add", "g")
        run(other, "commit", "-qm", "2")  # the lead landing while the worker runs
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertIn("protected-head-advanced", json.dumps(verdict["findings"]))

    def test_protected_head_rewrite_is_a_violation(self):
        other = self.make_other()
        launched = self.launch("clean", "--protect", str(other))
        run(other, "commit", "-q", "--amend", "-m", "rewritten")
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertIn("protected-head-moved", json.dumps(verdict["findings"]))

    def capsule_request(self, mode="clean", fetch_url="http://127.0.0.1:1/"):
        # The capsule clears the environment, so the fake's mode is baked into the script.
        tool = Path(self.tmp.name) / "fake-tool"
        (tool / "bin").mkdir(parents=True, exist_ok=True)
        script = self.fake.read_text().replace('os.environ.get("FAKE_MODE", "clean")', repr(mode))
        script = script.replace("__FETCH_URL__", fetch_url)
        (tool / "bin" / "traecli").write_text(script)
        (tool / "bin" / "traecli").chmod(0o755)
        req = Path(self.tmp.name) / "capsule.json"
        req.write_text(json.dumps({"name": "w", "layers": [{"kind": "base"},
                                   {"kind": "tree", "name": "fake", "source": str(tool), "path": ["bin"]}],
                                   "policy": {"net": "host"}}))
        os.environ["CORENIUS_CAPSULES"] = str(Path(self.tmp.name) / "capsules")
        os.environ["CORENIUS_TRAECLI"] = "traecli"
        return str(req)

    def test_worker_runs_inside_a_capsule(self):
        launched = self.launch("clean", "--capsule", self.capsule_request("capsule_clean"))
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "DEGRADED", verdict)
        self.assertFalse(verdict["ready_for_review"])
        self.assertIn("no-commit-after-base", verdict["not_ready"])
        run_dir = Path(launched["run_dir"])
        self.assertEqual((Path(launched["worktree"]) / "out.md").read_text(), "result\n")
        prompt = (run_dir / "prompt.md").read_text()
        self.assertIn("/workspace", prompt)
        self.assertNotIn(launched["worktree"], prompt)
        record = json.loads((run_dir / "capsule-run.json").read_text())
        self.assertEqual((record["net"], record["exit_code"]), ("host", 0))
        self.assertEqual(json.loads((run_dir / "run.json").read_text())["sandbox"], "danger-full-access (capsule)")

    def test_network_off_is_enforced_inside_a_capsule(self):
        class Handler(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                self.send_response(200)
                self.end_headers()
                self.wfile.write(b"ok")

            def log_message(self, format, *args):
                return

        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            url = f"http://127.0.0.1:{server.server_port}/"
            launched = self.launch("clean", "--capsule", self.capsule_request("capsule_fetch", url), "--network", "off")
            verdict = self.supervise(launched)
            self.assertEqual((Path(launched["worktree"]) / "out.md").read_text(), "fetch failed\n")
            record = json.loads((Path(launched["run_dir"]) / "capsule-run.json").read_text())
            self.assertEqual(record["net"], "none")
            self.assertIn("network-command", json.dumps(verdict["findings"]))
        finally:
            server.shutdown()

    def test_capsule_worker_boundary_violation_is_still_detected(self):
        verdict = self.supervise(self.launch("extra", "--capsule", self.capsule_request("extra")))
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertIn("stray.txt", json.dumps(verdict["findings"]))

    def test_nonzero_exit_is_degraded(self):
        verdict = self.supervise(self.launch("fail"))
        self.assertEqual(verdict["verdict"], "DEGRADED")
        self.assertEqual(verdict["exit_code"], 3)
        self.assertEqual(verdict["missing_allowed_outputs"], ["out.md"])

    def test_stall_is_reported_and_running_returns_3(self):
        launched = self.launch("stall")
        result = self.supervise(launched, max_wait=3)
        self.assertEqual(result["verdict"], "RUNNING")
        self.assertEqual(self.last_code, worker.RUNNING)
        self.assertIn("stall", json.dumps(result["findings"]))
        self.assertFalse((Path(launched["run_dir"]) / "verdict.json").exists())
        pid = json.loads((Path(launched["run_dir"]) / "run.json").read_text())["pid"]
        os.killpg(os.getpgid(pid), 15)

    def test_supervise_waits_for_process_not_for_output(self):
        # Output file appears immediately; the process runs 0.5s longer.
        launched = self.launch("clean")
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN")
        self.assertGreaterEqual(verdict["events"], 3)
        self.assertGreater(verdict["disk_usage"]["worktree_bytes"], 0)
        self.assertGreater(verdict["disk_usage"]["git_dir_bytes"], 0)
        self.assertGreater(verdict["disk_usage"]["worker_repo_bytes"], verdict["disk_usage"]["worktree_bytes"])
        self.assertGreater(verdict["disk_usage"]["run_dir_bytes"], 0)
        self.assertIn("caches", verdict["disk_usage"])

    def test_existing_worktree_is_refused(self):
        self.launch("clean")
        os.environ["FAKE_MODE"] = "clean"
        self._main(["launch", "--task", str(self.task), "--name", "t1", "--repo", str(self.repo)])
        self.assertEqual(self.last_code, 64)

    def test_missing_task_file_is_a_clean_error(self):
        self._main(["launch", "--task", str(self.task) + ".nope", "--name", "x", "--repo", str(self.repo)])
        self.assertEqual(self.last_code, 64)
        self.assertFalse((self.repo.parent / "corenius-worker-x").exists())

    # --- F12: --sandbox insula (read-only worktree, allowed paths bound writable) ---

    def test_sandbox_insula_needs_a_capsule(self):
        os.environ["FAKE_MODE"] = "clean"
        self._main(["launch", "--task", str(self.task), "--name", "s1", "--allow", "out.md",
                    "--repo", str(self.repo), "--sandbox", "insula"])
        self.assertEqual(self.last_code, 64)
        self.assertFalse((self.repo.parent / "corenius-worker-s1").exists())

    def test_sandbox_insula_worker_can_commit_with_private_git_metadata(self):
        launched = self.launch("clean", "--capsule", self.capsule_request("clean"), "--sandbox", "insula")

        verdict = self.supervise(launched)
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())

        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertTrue(verdict["ready_for_review"])
        self.assertEqual((Path(launched["worktree"]) / "out.md").read_text(), "x")
        self.assertEqual(run(self.repo, "rev-parse", launched["branch"]).strip(), verdict["candidate"]["commit"])
        self.assertEqual(run_data["git_dir_layout"], "embedded")
        self.assertTrue((Path(launched["worktree"]) / ".git").is_dir())

    def test_sandbox_binds_precreate_missing_allowed_paths(self):
        tree = Path(self.tmp.name) / "tree"
        (tree / "tools").mkdir(parents=True)
        (tree / "README").write_text("x\n")
        binds, created = worker.sandbox_binds(tree, ["README", "tools/agents/new.py", "scratch/"])
        self.assertEqual([dest for _, dest in binds],
                         ["/workspace/README", "/workspace/tools/agents/new.py", "/workspace/scratch"])
        self.assertEqual(created, [tree / "tools" / "agents", tree / "tools" / "agents" / "new.py", tree / "scratch"])
        self.assertTrue((tree / "tools" / "agents" / "new.py").is_file())
        self.assertTrue((tree / "scratch").is_dir())
        self.assertEqual((tree / "README").read_text(), "x\n")
        with self.assertRaises(worker.WorkerError):
            worker.sandbox_binds(tree, ["../escape"])
        with self.assertRaises(worker.WorkerError):
            worker.sandbox_binds(tree, ["/abs"])

    def test_remove_untouched_keeps_what_the_worker_wrote(self):
        tree = Path(self.tmp.name) / "tree2"
        tree.mkdir()
        _, created = worker.sandbox_binds(tree, ["a/kept.py", "b/empty.py", "c/"])
        (tree / "a" / "kept.py").write_text("print(1)\n")
        removed = worker.remove_untouched(created)
        self.assertEqual(sorted(removed), sorted([str(tree / "b" / "empty.py"), str(tree / "b"), str(tree / "c")]))
        self.assertTrue((tree / "a" / "kept.py").exists())
        self.assertFalse((tree / "b").exists())
        self.assertFalse((tree / "c").exists())


if __name__ == "__main__":
    unittest.main()
