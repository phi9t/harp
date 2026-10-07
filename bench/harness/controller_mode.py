"""Controller-backed benchmark mode for deterministic single-ticket proof.

The benchmark supplies disposable task fixtures and observes the result. The
SQLite controller owns admission, runtime dispatch, exact candidate
verification, acceptance and protected-ref landing.
"""

from __future__ import annotations

import base64
import hashlib
import io
import json
import os
import re
import shutil
import signal
import socket
import subprocess
import tarfile
import tempfile
import time
import urllib.error
import urllib.request
import uuid
from dataclasses import dataclass
from pathlib import Path

import fleet
import tasklib

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


CONTROLLER_SCHEMA = "corenius.bench.controller.v1"
PROJECT = "controller-bench"
EXECUTION_SOURCE_PATHS = [
    "tools/temp_root.py",
    "bench/harness/controller_mode.py",
    "bench/harness/run.py",
    "bench/harness/results.py",
    "bench/harness/tasklib.py",
    "bench/harness/fleet.py",
]
KATA = Path(os.environ.get(
    "CORENIUS_KATA_PINNED_BINARY",
    "/data02/home/philip.yang/workspace/corenius-codex-kata-evaluation/.scratch/team-control-plane/kata17/kata-pinned",
))
KATA_SHA256 = "8b0f05b1071f5e7a2b8173c9e5ef358d550a601cebd8c9d9cae6e45975e837f6"

PARTIAL_CONTRACTS = {
    "t1": {
        "allow": ["strings_util.py"],
        "selected_tests": ["T.test_title_keeps_small_words_lower_and_collapses_spaces"],
        "check": (
            "import strings_util\n"
            "assert strings_util.title_case('the  lord of the rings') == 'The Lord of the Rings'\n"
        ),
    },
    "t2": {
        "allow": ["numbers_util.py"],
        "selected_tests": ["T.test_clamp_both_ends"],
        "check": (
            "import numbers_util\n"
            "assert numbers_util.clamp(5, 0, 3) == 3\n"
            "assert numbers_util.clamp(-1, 0, 3) == 0\n"
            "assert numbers_util.clamp(2, 0, 3) == 2\n"
        ),
    },
}

WORKER_SCRIPT = r"""
import json
import pathlib
import shutil
import subprocess
import sys
import time

sys.dont_write_bytecode = True
scratch = pathlib.Path("/scratch")
workspace = pathlib.Path("/workspace")
sys.path.insert(0, str(workspace))
deadline = time.monotonic() + 30
while not (scratch / "action.json").exists():
    if time.monotonic() > deadline:
        raise SystemExit("controller-bench action deadline")
    time.sleep(0.02)

action = json.loads((scratch / "action.json").read_text())
if action.get("wait_for_useful_start"):
    (scratch / "waiting_for_useful_start").write_text("ready\n")
    while not (scratch / "start_useful").exists():
        if time.monotonic() > deadline:
            raise SystemExit("controller-bench useful-start deadline")
        time.sleep(0.02)

def git(*args):
    return subprocess.check_output(
        [
            "/usr/bin/git",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-C",
            "/workspace",
            *args,
        ],
        text=True,
    ).strip()

useful_start_monotonic_ns = time.monotonic_ns()
useful_start_time_ns = time.time_ns()
useful_steps = []
changed = []
for rel, content in action.get("files", {}).items():
    path = pathlib.Path(rel)
    if path.is_absolute() or ".." in path.parts:
        raise SystemExit("unsafe fixture path")
    target = workspace / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content)
    changed.append(rel)
    useful_steps.append({"kind": "write", "path": rel})

if action.get("check"):
    repeat = int(action.get("check_repeat", 1))
    for _ in range(repeat):
        exec(action["check"], {})
    useful_steps.append({"kind": "check", "repeat": repeat})
for pycache in workspace.rglob("__pycache__"):
    shutil.rmtree(pycache)

if changed:
    git("add", "--", *changed)
    git(
        "-c",
        "user.name=Controller bench worker",
        "-c",
        "user.email=controller-bench@invalid",
        "commit",
        "-m",
        "controller bench " + action["ticket"],
    )
    useful_steps.append({"kind": "commit", "paths": changed})
useful_end_monotonic_ns = time.monotonic_ns()
useful_end_time_ns = time.time_ns()

(scratch / "ready.json").write_text(
    json.dumps({
        "ticket": action["ticket"],
        "commit": git("rev-parse", "HEAD"),
        "changed": changed,
        "useful_interval": {
            "start_monotonic_ns": useful_start_monotonic_ns,
            "end_monotonic_ns": useful_end_monotonic_ns,
            "start_time_ns": useful_start_time_ns,
            "end_time_ns": useful_end_time_ns,
            "steps": useful_steps,
        },
    })
)
while not (scratch / "finish").exists():
    time.sleep(0.02)
"""


@dataclass
class ManagedProcess:
    name: str
    process: subprocess.Popen
    log: Path
    start_ticks: str

    def record(self) -> dict:
        return {
            "name": self.name,
            "pid": self.process.pid,
            "start_ticks": self.start_ticks,
            "exit": self.process.poll(),
            "log": str(self.log),
        }


def pin(path: Path | str) -> dict:
    path = Path(path).resolve()
    return {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def write_json(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    path.chmod(0o600)


def wait(description: str, predicate, seconds: float = 30.0):
    deadline = time.monotonic() + seconds
    last = None
    while time.monotonic() < deadline:
        try:
            value = predicate()
            if value:
                return value
            last = value
        except (ConnectionRefusedError, FileNotFoundError, TimeoutError, urllib.error.URLError, socket.timeout) as exc:
            last = exc
        time.sleep(0.05)
    raise TimeoutError(f"{description}: {last!r}")


def port() -> int:
    with socket.socket() as stream:
        stream.bind(("127.0.0.1", 0))
        return int(stream.getsockname()[1])


def fixed_git(path: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess:
    proc = subprocess.run(
        [
            "/usr/bin/git",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-C",
            str(path),
            *args,
        ],
        env={
            "PATH": "/usr/bin:/bin",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": "/dev/null",
            "GIT_NO_REPLACE_OBJECTS": "1",
            "GIT_TERMINAL_PROMPT": "0",
        },
        capture_output=True,
        text=True,
    )
    if check and proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip() or f"git {' '.join(args)} failed")
    return proc


def checkout_commit(repo: Path, commit: str, ws: Path) -> None:
    ws.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(
        [
            "/usr/bin/git",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-C",
            str(repo),
            "archive",
            "--format=tar",
            commit,
        ],
        env={
            "PATH": "/usr/bin:/bin",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": "/dev/null",
            "GIT_NO_REPLACE_OBJECTS": "1",
            "GIT_TERMINAL_PROMPT": "0",
        },
        capture_output=True,
        text=False,
    )
    if proc.returncode != 0:
        stderr = proc.stderr.decode(errors="replace").strip()
        raise RuntimeError(stderr or f"git archive {commit} failed")
    extract_git_archive_safely(proc.stdout, ws)


def extract_git_archive_safely(data: bytes, ws: Path) -> None:
    ws.mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:") as archive:
        members = archive.getmembers()
        total_bytes = 0
        if len(members) > 1024:
            raise RuntimeError("archive member bound exceeded")
        for member in members:
            if not member.name or member.name == ".":
                raise RuntimeError(f"unsafe archive member path: {member.name}")
            name = Path(member.name)
            if name.is_absolute() or "" in name.parts or ".." in name.parts:
                raise RuntimeError(f"unsafe archive member path: {member.name}")
            target = (ws / name).resolve()
            try:
                target.relative_to(ws.resolve())
            except ValueError as exc:
                raise RuntimeError(f"unsafe archive member path: {member.name}") from exc
            if member.isdir():
                continue
            if member.isfile():
                total_bytes += member.size
                if total_bytes > 16 * 1024 * 1024:
                    raise RuntimeError("archive byte bound exceeded")
                continue
            raise RuntimeError(f"unsupported archive member type: {member.name}")
        for member in members:
            target = ws / Path(member.name)
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            source = archive.extractfile(member)
            if source is None:
                raise RuntimeError(f"archive member content unavailable: {member.name}")
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open("wb") as stream:
                shutil.copyfileobj(source, stream)


def check_pin(record: dict) -> None:
    if sha256_bytes(Path(record["path"]).read_bytes()) != record["sha256"]:
        raise RuntimeError(f"pinned tool digest changed: {record['path']}")


def workspace_view_identity(root: Path) -> dict:
    entries = []
    for path in sorted(root.rglob("*")):
        rel = path.relative_to(root).as_posix()
        stat = path.lstat()
        if path.is_symlink():
            raise RuntimeError(f"unsupported verifier view symlink: {rel}")
        if path.is_dir():
            entries.append({"path": rel, "kind": "dir", "mode": stat.st_mode & 0o777})
            continue
        if path.is_file():
            entries.append({
                "path": rel,
                "kind": "file",
                "mode": stat.st_mode & 0o777,
                "bytes": stat.st_size,
                "sha256": sha256_bytes(path.read_bytes()),
            })
            continue
        raise RuntimeError(f"unsupported verifier view file type: {rel}")
    document = {"schema_version": "corenius.bench.verifier-view.v1", "entries": entries}
    return {"digest": sha256_bytes(json.dumps(document, sort_keys=True, separators=(",", ":")).encode()), "document": document}


def verify_runtime_manifest(runtime: dict) -> None:
    root = Path(runtime["root"])
    manifest = Path(runtime["manifest"]["path"])
    if sha256_bytes(manifest.read_bytes()) != runtime["manifest"]["sha256"]:
        raise RuntimeError("verifier runtime manifest digest changed")
    expected = json.loads(manifest.read_text())
    actual = runtime_manifest(root, manifest.parent / "independent-runtime-manifest-check.json")
    if actual["sha256"] != runtime["manifest"]["sha256"]:
        raise RuntimeError("verifier runtime manifest mismatch")
    if expected.get("schema_version") != "corenius.verifier-runtime.v1":
        raise RuntimeError("verifier runtime manifest schema changed")


def run_pinned_complete_suite(ws: Path, suite: dict, runtime: dict, tools: dict, host_root: Path, name: str) -> dict:
    verify_runtime_manifest(runtime)
    for key in ["python", "bubblewrap"]:
        check_pin(tools[key])
    check_pin({"path": suite["path"], "sha256": suite["sha256"]})
    suite_doc = complete_suite_entry("m01-disjoint-utils", "t2", suite, host_root)
    suite_digest = sha256_bytes(json.dumps(suite_doc, sort_keys=True, separators=(",", ":")).encode())
    started = time.time()
    proc = subprocess.run(
        [
            tools["bubblewrap"]["path"],
            "--unshare-all",
            "--new-session",
            "--die-with-parent",
            "--as-pid-1",
            "--clearenv",
            "--setenv", "TMPDIR", "/run/corenius-tmp",
            "--ro-bind",
            runtime["root"],
            "/usr",
            "--symlink",
            "usr/bin",
            "/bin",
            "--symlink",
            "usr/lib",
            "/lib",
            "--symlink",
            "usr/lib64",
            "/lib64",
            "--proc",
            "/proc",
            "--dev",
            "/dev",
            "--tmpfs",
            "/run/corenius-tmp",
            "--dir",
            "/home",
            "--dir",
            "/home/worker",
            "--setenv",
            "HOME",
            "/home/worker",
            "--setenv",
            "PATH",
            "/usr/bin:/bin",
            "--bind",
            str(ws),
            "/workspace",
            "--tmpfs",
            "/scratch",
            "--chdir",
            "/workspace",
            "--ro-bind",
            suite["path"],
            "/run/verification.py",
            tools["python"]["path"],
            "/run/verification.py",
            str(host_root),
            suite["protected_test_sha256"],
        ],
        env={},
        capture_output=True,
        text=True,
        timeout=120,
    )
    elapsed = round(time.time() - started, 3)
    log = {
        "name": name,
        "argv": [tools["python"]["path"], "/run/verification.py"],
        "args": [str(Path(suite["path"]).parent), suite["protected_test_sha256"]],
        "exit": proc.returncode,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
        "suite": suite_doc,
        "suite_digest": suite_digest,
        "tools": {"python": tools["python"], "bubblewrap": tools["bubblewrap"]},
        "runtime": {"manifest": runtime["manifest"]["sha256"], "root": runtime["root"]},
        "seconds": elapsed,
    }
    return {
        "passed": proc.returncode == 0,
        "rc": proc.returncode,
        "seconds": elapsed,
        "tail": "\n".join((proc.stdout + proc.stderr).splitlines()[-25:]),
        "log": log,
        "suite": {"id": suite_doc["id"], "digest": suite_digest},
    }


def independent_verify_commit(task: tasklib.Task, repo: Path, commit: str, suite: dict, runtime: dict, tools: dict,
                              root: Path, name: str,
                              tamper_tests: bool = False) -> dict:
    tree = fixed_git(repo, "rev-parse", "--verify", f"{commit}^{{tree}}").stdout.strip()
    ws = root / "independent-verifier" / name / "workspace"
    checkout_commit(repo, commit, ws)
    if tamper_tests:
        tests = ws / "tests/test_utils.py"
        tests.parent.mkdir(parents=True, exist_ok=True)
        tests.write_text("import unittest\n\nclass T(unittest.TestCase):\n    def test_tampered(self):\n        self.assertTrue(True)\n")
    source_view = workspace_view_identity(ws)
    verdict = run_pinned_complete_suite(ws, suite, runtime, tools, root, name)
    verifier_view = workspace_view_identity(ws)
    evidence_root = root / "independent-verifier" / name
    log_path = evidence_root / "verify-log.json"
    tail_path = evidence_root / "verify-tail.txt"
    write_json(log_path, verdict["log"])
    tail_path.write_text(verdict["tail"])
    tail_path.chmod(0o600)
    return {
        "source_commit": commit,
        "source_tree": tree,
        "source_view": source_view,
        "verifier_view": verifier_view,
        "commit": commit,
        "verdict": "pass" if verdict["passed"] else "fail",
        "rc": verdict["rc"],
        "seconds": verdict["seconds"],
        "tail_sha256": sha256_bytes(verdict["tail"].encode()),
        "tail": str(tail_path),
        "log": {"path": str(log_path), "sha256": pin(log_path)["sha256"]},
        "suite": verdict["suite"],
        "script": {"path": suite["path"], "sha256": suite["sha256"]},
        "runtime": {"status": "pinned", "root": runtime["root"], "manifest": runtime["manifest"]},
        "tools": {"python": tools["python"], "bubblewrap": tools["bubblewrap"]},
        "tampered_tests_before_restore": tamper_tests,
    }


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_root_parent() -> Path:
    selected = os.environ.get("TMPDIR")
    if not selected:
        raise RuntimeError("Run with tools/temp_root.py or set an owned TMPDIR")
    parent = Path(selected).resolve()
    forbidden = Path("/") / "tmp"
    if parent == forbidden or forbidden in parent.parents:
        raise RuntimeError("TMPDIR must be outside the system temporary directory")
    parent.mkdir(parents=True, exist_ok=True)
    return parent


def copy_runtime_path(source: Path, runtime_root: Path) -> Path:
    source = Path(source)
    source_bytes = source.resolve() if source.is_symlink() else source
    text = str(source)
    if text.startswith("/usr/"):
        rel = source.relative_to("/usr")
    elif text.startswith("/lib64/"):
        rel = Path("lib64") / source.relative_to("/lib64")
    elif text.startswith("/lib/"):
        rel = Path("lib") / source.relative_to("/lib")
    else:
        raise RuntimeError(f"runtime dependency outside supported roots: {source}")
    dest = runtime_root / rel
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists() and sha256_bytes(dest.read_bytes()) == sha256_bytes(source_bytes.read_bytes()):
        return dest
    shutil.copy2(source_bytes, dest)
    return dest


def selected_python_layout(python: Path) -> dict:
    proc = subprocess.run(
        [
            str(python),
            "-I",
            "-S",
            "-c",
            (
                "import json,sysconfig;"
                "print(json.dumps({"
                "'stdlib':sysconfig.get_path('stdlib'),"
                "'destshared':sysconfig.get_config_var('DESTSHARED') or ''"
                "},sort_keys=True))"
            ),
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout)


def ldd_dependencies(paths: list[Path]) -> list[Path]:
    found: set[Path] = set()
    for path in paths:
        proc = subprocess.run(["/usr/bin/ldd", str(path)], capture_output=True, text=True, check=True)
        for line in proc.stdout.splitlines():
            parts = line.strip().split()
            candidates = []
            if "=>" in parts:
                idx = parts.index("=>")
                if idx + 1 < len(parts):
                    candidates.append(parts[idx + 1])
            elif parts and parts[0].startswith("/"):
                candidates.append(parts[0])
            for item in candidates:
                candidate = Path(item)
                if item.startswith(("/lib/", "/lib64/", "/usr/lib/")) and candidate.is_file():
                    found.add(candidate)
    return sorted(found)


def runtime_manifest(runtime_root: Path, manifest_path: Path) -> dict:
    entries = []
    for path in sorted(runtime_root.rglob("*")):
        rel = path.relative_to(runtime_root).as_posix()
        stat = path.lstat()
        if path.is_symlink():
            target = os.readlink(path)
            if Path(target).is_absolute() or ".." in Path(rel).parent.joinpath(target).parts:
                raise RuntimeError(f"escaping runtime symlink refused: {rel} -> {target}")
            entries.append({"path": rel, "kind": "symlink", "target": target})
        elif path.is_file():
            data = path.read_bytes()
            entries.append({"path": rel, "kind": "regular", "bytes": stat.st_size, "sha256": sha256_bytes(data)})
    document = {"schema_version": "corenius.verifier-runtime.v1", "entries": entries}
    manifest_path.write_text(json.dumps(document, sort_keys=True, separators=(",", ":")) + "\n")
    manifest_path.chmod(0o600)
    return {"path": str(manifest_path), "sha256": sha256_bytes(manifest_path.read_bytes()), "entries": len(entries)}


def build_verifier_runtime(root: Path, python_pin: dict) -> dict:
    runtime_root = root / "runtime-snapshot" / "usr"
    runtime_root.mkdir(parents=True, exist_ok=True)
    python = Path(python_pin["path"])
    python_dest = copy_runtime_path(python, runtime_root)
    if not python_dest.is_file() or sha256_bytes(python_dest.read_bytes()) != python_pin["sha256"]:
        raise RuntimeError(f"runtime snapshot missing pinned interpreter: {python}")
    layout = selected_python_layout(python)
    stdlib = Path(layout["stdlib"]).resolve()
    shutil.copytree(stdlib, runtime_root / stdlib.relative_to("/usr"), dirs_exist_ok=True,
                    ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
    dynload = Path(layout["destshared"] or stdlib / "lib-dynload").resolve()
    if dynload.exists():
        shutil.copytree(dynload, runtime_root / dynload.relative_to("/usr"), dirs_exist_ok=True,
                        ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
    shared_inputs = [python]
    if dynload.exists():
        shared_inputs.extend(path for path in dynload.glob("*.so") if path.is_file())
    for dep in ldd_dependencies(shared_inputs):
        copy_runtime_path(dep, runtime_root)
    ambient = root / "ambient-runtime-source"
    ambient.mkdir(mode=0o700, parents=True, exist_ok=True)
    control_source = ambient / "value.txt"
    control_source.write_text("snapshotted\n")
    control = runtime_root / "share/corenius-runtime-control/value.txt"
    control.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(control_source, control)
    (ambient / "ambient_only.py").write_text("raise AssertionError('ambient source imported')\n")
    control_source.write_text("ambient changed after snapshot\n")
    manifest = runtime_manifest(runtime_root, root / "runtime-manifest.json")
    return {
        "root": str(runtime_root),
        "manifest": manifest,
        "control": {"path": str(control), "sha256": sha256_bytes(control.read_bytes())},
    }


def git_path_list(*args: str) -> list[str]:
    proc = subprocess.run(
        ["/usr/bin/git", "-C", str(tasklib.ROOT), *args],
        capture_output=True,
        check=True,
    )
    return [item.decode() for item in proc.stdout.split(b"\0") if item]


def source_content_manifest() -> dict:
    paths = set(EXECUTION_SOURCE_PATHS)
    paths.update(git_path_list("diff", "--name-only", "-z", "HEAD", "--"))
    paths.update(git_path_list("diff", "--cached", "--name-only", "-z", "--"))
    paths.update(git_path_list("ls-files", "--others", "--exclude-standard", "-z"))
    entries = []
    for rel in sorted(paths):
        path = tasklib.ROOT / rel
        if path.is_file():
            data = path.read_bytes()
            entries.append({"path": rel, "kind": "file", "bytes": len(data), "sha256": sha256_bytes(data)})
        else:
            entries.append({"path": rel, "kind": "missing"})
    digest = sha256_bytes(json.dumps(entries, sort_keys=True, separators=(",", ":")).encode())
    return {"schema_version": "corenius.bench.source-manifest.v1", "root": str(tasklib.ROOT), "digest": digest,
            "entries": entries}


def source_identity() -> dict:
    status = fixed_git(tasklib.ROOT, "status", "--porcelain", "--untracked-files=all").stdout
    diff = subprocess.run(
        ["/usr/bin/git", "-C", str(tasklib.ROOT), "diff", "--binary", "HEAD"],
        capture_output=True,
        text=False,
        check=True,
    ).stdout
    manifest = source_content_manifest()
    return {
        "head": fixed_git(tasklib.ROOT, "rev-parse", "HEAD").stdout.strip(),
        "tree": fixed_git(tasklib.ROOT, "rev-parse", "HEAD^{tree}").stdout.strip(),
        "dirty": bool(status.strip()),
        "status_sha256": sha256_bytes(status.encode()),
        "diff_sha256": sha256_bytes(diff),
        "content_manifest": manifest,
    }


def build_controller(root: Path) -> dict:
    target = tasklib.ROOT / ".scratch" / "controller-backed-bench" / "target"
    target.mkdir(parents=True, exist_ok=True)
    log = root / "controller-build.log"
    started = time.time()
    with log.open("ab") as stream:
        proc = subprocess.run(
            [
                "cargo",
                "build",
                "--manifest-path",
                str(tasklib.ROOT / "crates/sqlite-controller/Cargo.toml"),
                "--target-dir",
                str(target),
                "--locked",
                "--offline",
                "--bins",
            ],
            stdout=stream,
            stderr=subprocess.STDOUT,
        )
    elapsed = round(time.time() - started, 3)
    controller = target / "debug/corenius-sqlite-controller"
    supervisor = target / "debug/corenius-attempt-supervisor"
    gate = target / "debug/corenius-attempt-gate"
    record = {
        "command": "cargo build --manifest-path crates/sqlite-controller/Cargo.toml --target-dir .scratch/controller-backed-bench/target --locked --offline --bins",
        "exit": proc.returncode,
        "seconds": elapsed,
        "log": str(log),
    }
    if proc.returncode != 0:
        raise RuntimeError(f"controller build failed; see {log}")
    record.update({
        "controller": pin(controller),
        "supervisor": pin(supervisor),
        "attempt_gate": pin(gate),
    })
    return record


def write_partial_suite(path: Path, task: tasklib.Task) -> dict:
    protected = (task.path / "tests/test_utils.py").read_bytes()
    encoded = base64.b64encode(protected).decode()
    selected = {ticket: data["selected_tests"][0] for ticket, data in PARTIAL_CONTRACTS.items()}
    script = f"""import base64
import hashlib
import importlib.util
import pathlib
import sys
import unittest

workspace = pathlib.Path("/workspace")
sys.path.insert(0, str(workspace))
ticket = sys.argv[1]
host_root = pathlib.Path(sys.argv[2])
expected_digest = sys.argv[3]
selected = {json.dumps(selected, sort_keys=True)}[ticket]
assert not host_root.exists(), "controller root must be absent from verifier"
assert "CapEff:\\t0000000000000000" in pathlib.Path("/proc/self/status").read_text()
control = pathlib.Path("/usr/share/corenius-runtime-control/value.txt")
assert control.read_text() == "snapshotted\\n", "verifier consumed ambient runtime instead of snapshot"
try:
    control.write_text("verifier wrote runtime")
    raise AssertionError("verifier runtime snapshot is writable")
except OSError:
    pass
try:
    __import__("ambient_only")
    raise AssertionError("omitted ambient module loaded")
except ModuleNotFoundError:
    pass
test_bytes = base64.b64decode({encoded!r})
assert hashlib.sha256(test_bytes).hexdigest() == expected_digest
target = workspace / "tests" / "test_utils.py"
target.parent.mkdir(parents=True, exist_ok=True)
target.write_bytes(test_bytes)
spec = importlib.util.spec_from_file_location("protected_test_utils", target)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
suite = unittest.defaultTestLoader.loadTestsFromName(selected, module)
result = unittest.TextTestRunner(verbosity=2).run(suite)
if result.wasSuccessful():
    print("PARTIAL_CONTRACT_PASS", ticket, selected, "PROTECTED_TESTS_RESTORED")
sys.exit(0 if result.wasSuccessful() else 1)
"""
    path.write_text(script)
    path.chmod(0o600)
    return {"path": str(path), "sha256": sha256_bytes(script.encode()), "protected_test_sha256": sha256_bytes(protected)}


def write_complete_suite(path: Path, task: tasklib.Task) -> dict:
    protected = (task.path / "tests/test_utils.py").read_bytes()
    encoded = base64.b64encode(protected).decode()
    selected = ["T.test_title_keeps_small_words_lower_and_collapses_spaces", "T.test_clamp_both_ends"]
    script = f"""import base64
import hashlib
import importlib.util
import pathlib
import sys
import unittest

workspace = pathlib.Path("/workspace")
sys.path.insert(0, str(workspace))
host_root = pathlib.Path(sys.argv[1])
expected_digest = sys.argv[2]
selected = {json.dumps(selected, sort_keys=True)}
assert not host_root.exists(), "controller root must be absent from verifier"
assert "CapEff:\\t0000000000000000" in pathlib.Path("/proc/self/status").read_text()
control = pathlib.Path("/usr/share/corenius-runtime-control/value.txt")
assert control.read_text() == "snapshotted\\n", "verifier consumed ambient runtime instead of snapshot"
try:
    control.write_text("verifier wrote runtime")
    raise AssertionError("verifier runtime snapshot is writable")
except OSError:
    pass
try:
    __import__("ambient_only")
    raise AssertionError("omitted ambient module loaded")
except ModuleNotFoundError:
    pass
test_bytes = base64.b64decode({encoded!r})
assert hashlib.sha256(test_bytes).hexdigest() == expected_digest
target = workspace / "tests" / "test_utils.py"
target.parent.mkdir(parents=True, exist_ok=True)
target.write_bytes(test_bytes)
spec = importlib.util.spec_from_file_location("protected_test_utils", target)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromName(name, module) for name in selected)
result = unittest.TextTestRunner(verbosity=2).run(suite)
if result.wasSuccessful():
    print("COMPLETE_CONTRACT_PASS", ",".join(selected), "PROTECTED_TESTS_RESTORED")
sys.exit(0 if result.wasSuccessful() else 1)
"""
    path.write_text(script)
    path.chmod(0o600)
    return {
        "path": str(path),
        "sha256": sha256_bytes(script.encode()),
        "protected_test_sha256": sha256_bytes(protected),
        "selected_tests": selected,
    }


def suite_entry(task_id: str, task_name: str, base: str, suite: dict, root: Path, ticket_id: str, suffix: str = "") -> dict:
    return {
        "id": f"{task_id}:{ticket_id}:partial{suffix}",
        "task": task_name,
        "base": base,
        "script": {"path": suite["path"], "sha256": suite["sha256"]},
        "args": [ticket_id, str(root), suite["protected_test_sha256"]],
    }


def complete_suite_entry(task_id: str, task_name: str, suite: dict, root: Path) -> dict:
    return {
        "id": f"{task_id}:complete",
        "task": task_name,
        "base": None,
        "script": {"path": suite["path"], "sha256": suite["sha256"]},
        "args": [str(root), suite["protected_test_sha256"]],
    }


def controller_reply(socket_path: Path, value: dict) -> dict:
    with socket.socket(socket.AF_UNIX) as stream:
        stream.settimeout(40)
        stream.connect(str(socket_path))
        stream.sendall(json.dumps(value).encode() + b"\n")
        return json.loads(stream.makefile("rb").readline())


def controller_request(socket_path: Path, value: dict) -> dict:
    reply = controller_reply(socket_path, value)
    if "error" in reply:
        raise RuntimeError(reply["error"])
    return reply["ok"]


def http_json(origin: str, method: str, path: str, token: str | None = None, body: dict | None = None) -> dict:
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = "Bearer " + token
    req = urllib.request.Request(
        origin + "/api/v1" + path,
        data=None if body is None else json.dumps(body).encode(),
        method=method,
        headers=headers,
    )
    with urllib.request.urlopen(req, timeout=5) as response:
        return json.load(response)


def extract_diff_paths(diff: str) -> list[str]:
    paths = []
    for line in diff.splitlines():
        match = re.match(r"diff --git a/(.+?) b/(.+)", line)
        if match:
            paths.append(match.group(2))
    return sorted(set(paths))


def ticket_overlay(ticket: fleet.Ticket) -> dict[str, str]:
    return {
        str(path.relative_to(ticket.path / "overlay")): path.read_text()
        for path in sorted((ticket.path / "overlay").rglob("*"))
        if path.is_file()
    }


def useful_interval(ticket: str, attempt: dict) -> dict:
    raw = attempt["ready"]["useful_interval"]
    return {
        "ticket": ticket,
        "attempt": attempt["grant"]["attempt"],
        "base": attempt["grant"]["base"],
        "candidate": attempt["candidate"]["candidate"],
        "commit": attempt["candidate"]["commit"],
        "start_monotonic_ns": raw["start_monotonic_ns"],
        "end_monotonic_ns": raw["end_monotonic_ns"],
        "start_time_ns": raw["start_time_ns"],
        "end_time_ns": raw["end_time_ns"],
        "steps": raw["steps"],
    }


def measured_overlap(first: dict, second: dict) -> dict:
    intervals = [first, second]
    start = max(entry["start_monotonic_ns"] for entry in intervals)
    end = min(entry["end_monotonic_ns"] for entry in intervals)
    seconds = round(max(0, end - start) / 1_000_000_000, 6)
    return {
        "basis": "worker-reported monotonic interval from file write/check/commit only",
        "seconds": seconds,
        "intervals": intervals,
    }


def public_grant(grant: dict) -> dict:
    return {k: grant[k] for k in ["attempt", "operation", "slot", "principal", "scope_epoch", "base", "brief"]}


def stopped_services(processes: list[ManagedProcess]) -> list[dict]:
    records = []
    for entry in reversed(processes):
        proc = entry.process
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5)
        records.append(entry.record())
    return list(reversed(records))


class ControllerRun:
    def __init__(self, task: tasklib.Task, ticket: fleet.Ticket, root: Path, build: dict, source: dict):
        self.task = task
        self.ticket = ticket
        self.root = root
        self.build = build
        self.source = source
        self.processes: list[ManagedProcess] = []
        self.operator_token = uuid.uuid4().hex + uuid.uuid4().hex
        self.supervisor_token = uuid.uuid4().hex + uuid.uuid4().hex
        self.kata_bootstrap = uuid.uuid4().hex + uuid.uuid4().hex
        self.kata_token = ""
        self.kata_origin = ""
        self.project_revision = 0

    @property
    def controller_socket(self) -> Path:
        return self.root / "controller.sock"

    def spawn(self, name: str, argv: list[Path | str], env: dict[str, str]) -> ManagedProcess:
        log = self.root / f"{name}.log"
        with log.open("ab") as stream:
            proc = subprocess.Popen(
                [str(a) for a in argv],
                cwd=self.root,
                env=env,
                stdout=stream,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            )
        start_ticks = Path(f"/proc/{proc.pid}/stat").read_text().rsplit(") ", 1)[1].split()[19]
        entry = ManagedProcess(name, proc, log, start_ticks)
        self.processes.append(entry)
        return entry

    def query(self, kind: str = "snapshot", args: dict | None = None, auth: dict | None = None) -> dict:
        return controller_request(
            self.controller_socket,
            {"auth": auth or {"principal": "operator", "token": self.operator_token},
             "project": PROJECT, "query": {"kind": kind, "args": args or {}}},
        )

    def command(self, kind: str, args: dict, auth: dict | None = None, key: str | None = None,
                allow_error: bool = False) -> dict:
        auth = auth or {"principal": "operator", "token": self.operator_token}
        idempotency_key = key or f"controller-bench-{kind}-{uuid.uuid4()}"
        last = None
        for _ in range(25):
            revision = 0 if kind == "create_project" else self.query()["revision"]
            reply = controller_reply(
                self.controller_socket,
                {"auth": auth,
                 "project": PROJECT,
                 "command": {
                     "kind": kind,
                     "expected_revision": revision,
                     "idempotency_key": idempotency_key,
                     "args": args,
                 }},
            )
            if "ok" in reply:
                return reply["ok"]
            last = RuntimeError(reply["error"])
            if not str(last).startswith("revision conflict:"):
                if allow_error:
                    return reply
                raise last
            time.sleep(0.02)
        raise RuntimeError(str(last))

    def setup_repositories(self, tickets: list[fleet.Ticket] | None = None) -> tuple[str, str]:
        tickets = tickets or [self.ticket]
        for name in ["authority", "attempts", "canonical", "seed", "intent", "kata"]:
            path = self.root / name
            path.mkdir(mode=0o700, parents=True, exist_ok=True)
        fleet.git(self.root / "canonical", "init", "--bare", "--template=")
        tasklib.materialize(self.task, self.root / "seed")
        fleet.git(self.root / "seed", "init", "-q", "-b", "main")
        fleet.git(self.root / "seed", "add", "-A")
        fleet.git(self.root / "seed", "commit", "-q", "-m", "m01 start")
        base = fleet.git(self.root / "seed", "rev-parse", "HEAD").stdout.strip()
        fleet.git(self.root / "seed", "push", str(self.root / "canonical"), f"{base}:refs/heads/protected")

        fleet.git(self.root / "intent", "init", "-q", "-b", "main")
        (self.root / "intent/spec.md").write_text((self.task.path / "instruction.md").read_text())
        for ticket in tickets:
            (self.root / "intent" / f"{ticket.id}.md").write_text(ticket.instruction)
        fleet.git(self.root / "intent", "add", ".")
        fleet.git(
            self.root / "intent",
            "-c",
            "user.name=Controller bench intent",
            "-c",
            "user.email=controller-bench@invalid",
            "commit",
            "-q",
            "-m",
            "m01 controller intent",
        )
        intent = fleet.git(self.root / "intent", "rev-parse", "HEAD").stdout.strip()
        return base, intent

    def setup_kata(self) -> dict:
        if not KATA.is_file() or pin(KATA)["sha256"] != KATA_SHA256:
            raise RuntimeError("pinned Kata 0.24.0 binary unavailable or hash changed")
        (self.root / "kata/config.toml").write_text(
            f'[auth]\ntoken="{self.kata_bootstrap}"\nrequire_token_identity=true\n'
        )
        (self.root / "kata/config.toml").chmod(0o600)
        kport = port()
        self.kata_origin = f"http://127.0.0.1:{kport}"
        env = {k: v for k, v in os.environ.items() if not k.startswith("KATA_")}
        env["KATA_HOME"] = str(self.root / "kata")
        self.spawn("kata", [KATA, "daemon", "start", "--foreground", "--listen", f"127.0.0.1:{kport}"], env)
        wait("Kata health", lambda: http_json(self.kata_origin, "GET", "/health", token=None), 20)
        token = http_json(
            self.kata_origin,
            "POST",
            "/tokens",
            token=self.kata_bootstrap,
            body={"actor": "corenius-controller", "name": "controller-bench"},
        )["plaintext"]
        self.kata_token = token
        (self.root / "kata.token").write_text(token)
        (self.root / "kata.token").chmod(0o600)
        project = http_json(self.kata_origin, "POST", "/projects", token=token, body={"name": self.root.name})["project"]
        instance = http_json(self.kata_origin, "GET", "/instance", token=token)["instance_uid"]
        return {"binary": pin(KATA), "origin": self.kata_origin, "instance_uid": instance, "project": project}

    def create_kata_issue(self, title: str, body: str, project: dict) -> dict:
        return http_json(
            self.kata_origin,
            "POST",
            f"/projects/{project['id']}/issues",
            token=self.kata_token,
            body={"title": title, "body": body, "force_new": True},
        )["issue"]

    def setup_policy(self, base: str, suite: dict, kata: dict, tickets: list[fleet.Ticket] | None = None,
                     complete_suite: dict | None = None) -> dict:
        tickets = tickets or [self.ticket]
        python = pin("/usr/bin/python3")
        verifier_runtime = build_verifier_runtime(self.root, python)
        supervisor_config = {
            "root": str(self.root / "attempts"),
            "profile": {"executable": "/usr/bin/python3", "args": ["-c", WORKER_SCRIPT], "codex": None},
        }
        write_json(self.root / "supervisor.json", supervisor_config)
        suites = []
        for ticket in tickets:
            suites.extend([
                suite_entry(self.task.id, f"{ticket.id}-noop", base, suite, self.root, ticket.id, ":noop"),
                suite_entry(self.task.id, ticket.id, base, suite, self.root, ticket.id),
            ])
        if complete_suite is not None:
            suites.append(complete_suite_entry(self.task.id, tickets[-1].id, complete_suite, self.root))
        policy = {
            "project": PROJECT,
            "root": str(self.root / "authority"),
            "attempts_root": str(self.root / "attempts"),
            "canonical": str(self.root / "canonical"),
            "protected_ref": "refs/heads/protected",
            "git": pin("/usr/bin/git"),
            "python": python,
            "bubblewrap": pin("/usr/bin/bwrap"),
            "verifier_runtime": {
                "root": verifier_runtime["root"],
                "manifest": {
                    "path": verifier_runtime["manifest"]["path"],
                    "sha256": verifier_runtime["manifest"]["sha256"],
                },
            },
            "suites": suites,
        }
        write_json(self.root / "git.json", policy)
        kata_config = {
            "schema_migration": "kata-v1",
            "controller_id": self.root.name,
            "actor": "corenius-controller",
            "api_schema_version": "0.24.0",
            "origin_instance_uid": kata["instance_uid"],
            "origin": kata["origin"],
            "token_file": str(self.root / "kata.token"),
            "project_id": kata["project"]["id"],
            "project_uid": kata["project"]["uid"],
            "repositories": {"intent": str(self.root / "intent")},
        }
        write_json(self.root / "kata.json", kata_config)
        return {
            "git_policy_sha256": pin(self.root / "git.json")["sha256"],
            "kata_config_sha256": pin(self.root / "kata.json")["sha256"],
            "supervisor_config_sha256": pin(self.root / "supervisor.json")["sha256"],
            "tools": {key: policy[key] for key in ["git", "python", "bubblewrap"]},
            "verifier_runtime": verifier_runtime,
            "suites": suites,
            "complete_suite": complete_suite,
        }

    def start_services(self) -> None:
        supervisor_env = {
            "PATH": "/usr/bin:/bin",
            "CORENIUS_SUPERVISOR_TOKEN": self.supervisor_token,
            "CORENIUS_GIT_CONFIG": str(self.root / "git.json"),
        }
        self.spawn(
            "supervisor",
            [self.build["supervisor"]["path"], self.root / "supervisor.json", self.root / "supervisor.sock"],
            supervisor_env,
        )
        wait("supervisor socket", lambda: (self.root / "supervisor.sock").exists(), 20)
        controller_env = {
            "PATH": "/usr/bin:/bin",
            "CORENIUS_OPERATOR_TOKEN": self.operator_token,
            "CORENIUS_SUPERVISOR_TOKEN": self.supervisor_token,
            "CORENIUS_SUPERVISOR_SOCKET": str(self.root / "supervisor.sock"),
            "CORENIUS_GIT_CONFIG": str(self.root / "git.json"),
            "CORENIUS_KATA_CONFIG": str(self.root / "kata.json"),
        }
        self.spawn(
            "controller",
            [self.build["controller"]["path"], self.root / "state.db", self.controller_socket],
            controller_env,
        )
        wait("controller socket", lambda: self.controller_socket.exists(), 20)
        wait("controller storage", lambda: self.query("storage_configuration"), 20)

    def create_project_and_lead(self, base: str, intent: str, tickets: list[fleet.Ticket] | None = None,
                                complete: bool = False) -> tuple[dict, dict]:
        tickets = tickets or [self.ticket]
        ticket_ids = [ticket.id for ticket in tickets]
        project = self.command("create_project", {"name": PROJECT, "base": base}, key="create-project")
        lead = self.command("replace_lead", {}, key="replace-lead")["result"]
        lead_auth = {"principal": lead["principal"], "token": lead["token"]}
        mission = {
            "objective": (
                f"Run complete {self.task.id} through the controller-backed benchmark path"
                if complete else f"Run {self.task.id} {self.ticket.id} through the controller-backed benchmark path"
            ),
            "plan": (
                "deterministic lead creates two Kata-backed tasks, admits two supervised workers from the same base, "
                "verifies and accepts both exact candidates, lands the first, observes stale refusal for the second, "
                "then admits one fresh integration attempt with fresh verification and acceptance"
                if complete else
                "deterministic lead creates one Kata-backed task, admits one supervised worker, verifies the exact candidate, accepts it and lands the disposable protected ref"
            ),
            "constraints": "no model calls; disposable repository only; protected verifier inputs restored by pinned suite",
            "intent": f"{self.task.id}:{','.join(ticket_ids)}:{intent}",
        }
        write_json(self.root / "mission.json", mission)
        mission_receipt = self.command("set_mission", mission, auth=lead_auth, key="set-mission")
        return lead_auth, {"project": project, "lead": {k: lead[k] for k in ["principal", "role_epoch"]}, "mission": mission_receipt}

    def create_controller_task(self, task_name: str, issue: dict, intent: str, lead_auth: dict,
                               ticket: fleet.Ticket | None = None) -> dict:
        ticket = ticket or self.ticket
        args = {
            "task": task_name,
            "item": issue["uid"],
            "expected_source_revision": issue["revision"],
            "source_files": {
                "storage": "intent",
                "mode": "git",
                "commit": intent,
                "paths": ["spec.md", f"{ticket.id}.md"],
            },
        }
        return self.command("create_kata_task", args, auth=lead_auth, key=f"create-{task_name}")

    def begin_attempt(self, task_name: str, lead_auth: dict, action: dict, key_prefix: str) -> dict:
        admission = self.command("admit", {"task": task_name}, auth=lead_auth, key=f"admit-{key_prefix}")
        grant = admission["result"]
        root = self.root / "attempts" / grant["attempt"]
        wait("attempt workspace", lambda: (root / "workspace/.git/objects").is_dir(), 30)
        write_json(root / "scratch/action.json", action)
        return {
            "admission": admission,
            "grant": grant,
            "root": str(root),
        }

    def release_useful_start(self, attempt: dict) -> None:
        root = Path(attempt["root"])
        (root / "scratch/start_useful").write_text("start\n")

    def wait_attempt_candidate(self, task_name: str, lead_auth: dict, attempt: dict, key_prefix: str) -> dict:
        root = Path(attempt["root"])
        ready = wait("worker useful checkpoint", lambda: json.loads((root / "scratch/ready.json").read_text()) if (root / "scratch/ready.json").exists() else None, 30)
        worker_auth = {"principal": attempt["grant"]["principal"], "token": attempt["grant"]["token"]}
        progress = self.command(
            "report_progress",
            {"task": task_name, "text": f"deterministic worker produced {ready['commit']} for {ready['ticket']}"},
            auth=worker_auth,
            key=f"progress-{key_prefix}",
        )
        candidate = self.command(
            "capture_candidate",
            {"report": f"deterministic worker commit {ready['commit']} for {self.task.id} {ready['ticket']}"},
            auth=worker_auth,
            key=f"capture-{key_prefix}",
        )
        return {**attempt, "ready": ready, "progress": progress, "candidate": candidate["result"]}

    def run_attempt(self, task_name: str, lead_auth: dict, action: dict, key_prefix: str | None = None) -> dict:
        key_prefix = key_prefix or task_name
        attempt = self.begin_attempt(task_name, lead_auth, action, key_prefix)
        return self.wait_attempt_candidate(task_name, lead_auth, attempt, key_prefix)

    def finish_attempt(self, attempt: str) -> dict:
        root = self.root / "attempts" / attempt
        (root / "scratch/finish").write_text("controller-bench deterministic attempt complete\n")
        return wait(
            "attempt exited",
            lambda: self.query() if any(a["id"] == attempt and a["state"] == "exited" for a in self.query()["attempts"]) else None,
            30,
        )

    def attempt_process_record(self, grant: dict) -> dict:
        record_path = self.root / "attempts" / f"{grant['operation']}.json"
        value = json.loads(record_path.read_text())
        return {
            "attempt": grant["attempt"],
            "operation": grant["operation"],
            "pid": value.get("pid"),
            "start_ticks": value.get("start_ticks"),
            "namespace_init": value.get("namespace_init"),
            "init_start_ticks": value.get("init_start_ticks"),
            "pid_namespace": value.get("pid_namespace"),
            "state": value.get("state"),
            "outcome": value.get("outcome"),
            "record": str(record_path),
        }

    def verify_accept_land(self, candidate: dict, lead_auth: dict, base: str, ticket: fleet.Ticket | None = None,
                           key_prefix: str = "fixed") -> tuple[dict, dict, dict, dict, dict]:
        ticket = ticket or self.ticket
        timings = {}
        started = time.time()
        comparison = self.command("compare_candidates", {"candidate": candidate["candidate"]}, auth=lead_auth, key=f"compare-{key_prefix}")
        timings["compare_seconds"] = round(time.time() - started, 3)
        diff_paths = extract_diff_paths(comparison["result"]["diff"])
        contamination = sorted(p for p in diff_paths if not fleet.allowed(p, ticket.allow))
        started = time.time()
        verification = self.command("verify_candidate", {"candidate": candidate["candidate"]}, auth=lead_auth, key=f"verify-{key_prefix}")
        timings["fixed_verify_seconds"] = round(time.time() - started, 3)
        if contamination:
            raise RuntimeError(f"controller candidate changed paths outside ticket allowlist: {contamination}")
        if verification["result"]["verdict"] != "pass":
            raise RuntimeError(f"controller partial verifier did not pass: {verification['result']}")
        started = time.time()
        acceptance = self.command(
            "accept_candidate",
            {"candidate": candidate["candidate"], "verification": verification["result"]["verification"]},
            auth=lead_auth,
            key=f"accept-{key_prefix}",
        )
        timings["accept_seconds"] = round(time.time() - started, 3)
        started = time.time()
        landing = self.command(
            "land_candidate",
            {"candidate": candidate["candidate"], "expected_trunk": base},
            auth=lead_auth,
            key=f"land-{key_prefix}",
        )
        timings["land_seconds"] = round(time.time() - started, 3)
        return comparison["result"], verification["result"], acceptance["result"], landing["result"], timings

    def verify_accept(self, candidate: dict, lead_auth: dict, ticket: fleet.Ticket, key_prefix: str) -> tuple[dict, dict, dict, dict]:
        timings = {}
        started = time.time()
        comparison = self.command("compare_candidates", {"candidate": candidate["candidate"]}, auth=lead_auth, key=f"compare-{key_prefix}")
        timings["compare_seconds"] = round(time.time() - started, 3)
        diff_paths = extract_diff_paths(comparison["result"]["diff"])
        contamination = sorted(p for p in diff_paths if not fleet.allowed(p, ticket.allow))
        if contamination:
            raise RuntimeError(f"controller candidate changed paths outside ticket allowlist: {contamination}")
        started = time.time()
        verification = self.command("verify_candidate", {"candidate": candidate["candidate"]}, auth=lead_auth, key=f"verify-{key_prefix}")
        timings["verify_seconds"] = round(time.time() - started, 3)
        if verification["result"]["verdict"] != "pass":
            raise RuntimeError(f"controller verifier did not pass: {verification['result']}")
        started = time.time()
        acceptance = self.command(
            "accept_candidate",
            {"candidate": candidate["candidate"], "verification": verification["result"]["verification"]},
            auth=lead_auth,
            key=f"accept-{key_prefix}",
        )
        timings["accept_seconds"] = round(time.time() - started, 3)
        return comparison["result"], verification["result"], acceptance["result"], timings

    def run(self) -> dict:
        started = time.time()
        setup_started = time.time()
        base, intent = self.setup_repositories()
        suite = write_partial_suite(self.root / "partial_verifier.py", self.task)
        kata = self.setup_kata()
        policy = self.setup_policy(base, suite, kata)
        self.start_services()
        lead_auth, control = self.create_project_and_lead(base, intent)
        setup_seconds = round(time.time() - setup_started, 3)
        flow_started = time.time()
        project = kata["project"]
        noop_issue = self.create_kata_issue(
            f"{self.task.id} {self.ticket.id} noop control",
            "Negative control: submit the base tree for the selected partial contract.",
            project,
        )
        fixed_issue = self.create_kata_issue(
            f"{self.task.id} {self.ticket.id}",
            self.ticket.instruction,
            project,
        )
        noop_task = f"{self.ticket.id}-noop"
        self.create_controller_task(noop_task, noop_issue, intent, lead_auth)
        phase_started = time.time()
        noop = self.run_attempt(noop_task, lead_auth, {"ticket": self.ticket.id, "files": {}, "check": ""})
        noop_attempt_seconds = round(time.time() - phase_started, 3)
        phase_started = time.time()
        noop_verification = self.command(
            "verify_candidate",
            {"candidate": noop["candidate"]["candidate"]},
            auth=lead_auth,
            key="verify-noop",
        )["result"]
        noop_verify_seconds = round(time.time() - phase_started, 3)
        phase_started = time.time()
        self.finish_attempt(noop["grant"]["attempt"])
        noop_finish_seconds = round(time.time() - phase_started, 3)

        overlay = {
            str(path.relative_to(self.ticket.path / "overlay")): path.read_text()
            for path in sorted((self.ticket.path / "overlay").rglob("*"))
            if path.is_file()
        }
        self.create_controller_task(self.ticket.id, fixed_issue, intent, lead_auth)
        phase_started = time.time()
        fixed = self.run_attempt(
            self.ticket.id,
            lead_auth,
            {"ticket": self.ticket.id, "files": overlay, "check": PARTIAL_CONTRACTS[self.ticket.id]["check"]},
        )
        fixed_attempt_seconds = round(time.time() - phase_started, 3)
        comparison, verification, acceptance, landing, authority_timings = self.verify_accept_land(fixed["candidate"], lead_auth, base)
        phase_started = time.time()
        final_snapshot = self.finish_attempt(fixed["grant"]["attempt"])
        fixed_finish_seconds = round(time.time() - phase_started, 3)
        inventory = self.query("workspace_inventory")
        events = self.query("events", {"after": 0, "limit": 100})
        final_ref = fleet.git(self.root / "canonical", "rev-parse", "refs/heads/protected").stdout.strip()
        flow_seconds = round(time.time() - flow_started, 3)
        cleanup_started = time.time()
        process_records = stopped_services(self.processes)
        cleanup_seconds = round(time.time() - cleanup_started, 3)
        attempt_records = [self.attempt_process_record(noop["grant"]), self.attempt_process_record(fixed["grant"])]
        timing = {
            "wall_seconds": round(time.time() - started, 3),
            "setup_seconds": setup_seconds,
            "controller_flow_seconds": flow_seconds,
            "attempt_seconds": round(noop_attempt_seconds + fixed_attempt_seconds, 3),
            "verify_seconds": round(noop_verify_seconds + authority_timings["fixed_verify_seconds"], 3),
            "noop_attempt_seconds": noop_attempt_seconds,
            "noop_verify_seconds": noop_verify_seconds,
            "noop_finish_seconds": noop_finish_seconds,
            "fixed_attempt_seconds": fixed_attempt_seconds,
            "fixed_verify_seconds": authority_timings["fixed_verify_seconds"],
            "fixed_finish_seconds": fixed_finish_seconds,
            "compare_seconds": authority_timings["compare_seconds"],
            "accept_seconds": authority_timings["accept_seconds"],
            "land_seconds": authority_timings["land_seconds"],
            "cleanup_seconds": cleanup_seconds,
        }
        partial = {
            "suite_id": verification["document"]["suite_id"],
            "suite_digest": verification["document"]["suite"],
            "selected_tests": PARTIAL_CONTRACTS[self.ticket.id]["selected_tests"],
            "protected_test_sha256": suite["protected_test_sha256"],
            "complete_original_test_file_restored": True,
            "noop_verdict": noop_verification["verdict"],
            "fixed_verdict": verification["verdict"],
        }
        orchestration_passed = (
            noop_verification["verdict"] == "fail"
            and verification["verdict"] == "pass"
            and acceptance["candidate"] == fixed["candidate"]["candidate"]
            and landing["state"] == "landed"
            and final_ref == fixed["candidate"]["commit"]
            and not comparison["diff"] == ""
            and not any(p for p in extract_diff_paths(comparison["diff"]) if not fleet.allowed(p, self.ticket.allow))
            and any(a["id"] == fixed["grant"]["attempt"] and a["state"] == "exited" for a in final_snapshot["attempts"])
        )
        return {
            "schema_version": CONTROLLER_SCHEMA,
            "mode": "deterministic",
            "task": self.task.id,
            "ticket": self.ticket.id,
            "run_root": str(self.root),
            "source": self.source,
            "build": self.build,
            "control": control,
            "roles": {"lead": control["lead"], "noop_attempt": public_grant(noop["grant"]), "fixed_attempt": public_grant(fixed["grant"])},
            "kata": {"binary": kata["binary"], "instance_uid": kata["instance_uid"], "project": kata["project"],
                     "issues": {"noop": noop_issue, "fixed": fixed_issue}, "config_sha256": policy["kata_config_sha256"]},
            "git": {"policy_sha256": policy["git_policy_sha256"], "canonical": str(self.root / "canonical"),
                    "protected_ref": "refs/heads/protected", "base": base, "final": final_ref},
            "supervisor": {"config_sha256": policy["supervisor_config_sha256"]},
            "runtime_closure": {
                "status": "pinned",
                "pinned": {
                    "verification_adapter": {"path": suite["path"], "sha256": suite["sha256"]},
                    "protected_test_sha256": suite["protected_test_sha256"],
                    "git": policy["tools"]["git"],
                    "python_executable": policy["tools"]["python"],
                    "bubblewrap": policy["tools"]["bubblewrap"],
                    "verifier_runtime": policy["verifier_runtime"],
                },
                "enforced": "sqlite-controller validates the pinned runtime manifest, ro-binds that private snapshot at /usr, and binds the runtime digest into verifier proof",
            },
            "partial_contract": partial,
            "noop": {"task": noop_task, "candidate": noop["candidate"], "verification": noop_verification},
            "candidate": fixed["candidate"],
            "comparison": {"changed_paths": extract_diff_paths(comparison["diff"]), "contamination": []},
            "verification": verification,
            "acceptance": acceptance,
            "landing": landing,
            "process_disposition": {"owned_attempts": "stopped", "services": "stopped", "processes": process_records,
                                    "attempts": attempt_records},
            "events_observed": {"count": len(events), "last_seq": events[-1]["seq"] if events else 0},
            "workspace_inventory": {
                "configured": inventory.get("configured"),
                "backend": inventory.get("backend"),
                "workspaces": len(inventory.get("workspaces", [])),
                "history": len(inventory.get("history", [])),
            },
            "timing": timing,
            "qualification_limits": [
                "single deterministic ticket only",
                "no live model calls",
                "no two-worker overlap, stale-base integration or permanent readiness claim",
            ],
            "orchestration_passed": orchestration_passed,
        }

    def run_complete(self, tickets: list[fleet.Ticket]) -> dict:
        tickets_by_id = {ticket.id: ticket for ticket in tickets}
        if set(tickets_by_id) != {"t1", "t2"}:
            raise RuntimeError("complete controller mode requires m01 tickets t1 and t2")

        started = time.time()
        setup_started = time.time()
        base, intent = self.setup_repositories(tickets)
        partial_suite = write_partial_suite(self.root / "partial_verifier.py", self.task)
        complete_suite = write_complete_suite(self.root / "complete_verifier.py", self.task)
        kata = self.setup_kata()
        policy = self.setup_policy(base, partial_suite, kata, tickets, complete_suite=complete_suite)
        self.start_services()
        lead_auth, control = self.create_project_and_lead(base, intent, tickets, complete=True)
        setup_seconds = round(time.time() - setup_started, 3)
        flow_started = time.time()
        project = kata["project"]

        issues = {
            ticket.id: self.create_kata_issue(f"{self.task.id} {ticket.id}", ticket.instruction, project)
            for ticket in tickets
        }
        for ticket in tickets:
            self.create_controller_task(ticket.id, issues[ticket.id], intent, lead_auth, ticket)

        begin_started = time.time()
        initial_attempts = {}
        for ticket in tickets:
            initial_attempts[ticket.id] = self.begin_attempt(
                ticket.id,
                lead_auth,
                {
                    "ticket": ticket.id,
                    "files": ticket_overlay(ticket),
                    "check": PARTIAL_CONTRACTS[ticket.id]["check"],
                    "check_repeat": 20_000,
                    "wait_for_useful_start": True,
                },
                key_prefix=f"initial-{ticket.id}",
            )
        for attempt in initial_attempts.values():
            root = Path(attempt["root"])
            wait("worker ready for useful release", lambda root=root: (root / "scratch/waiting_for_useful_start").exists(), 30)
        for attempt in initial_attempts.values():
            self.release_useful_start(attempt)
        initial = {
            ticket_id: self.wait_attempt_candidate(ticket_id, lead_auth, attempt, key_prefix=f"initial-{ticket_id}")
            for ticket_id, attempt in initial_attempts.items()
        }
        initial_attempt_wall_seconds = round(time.time() - begin_started, 3)
        if {initial["t1"]["grant"]["base"], initial["t2"]["grant"]["base"]} != {base}:
            raise RuntimeError("initial worker attempts did not start from the same canonical base")
        two_worker_snapshot = self.query()
        if two_worker_snapshot["occupied_slots"] != 2:
            raise RuntimeError(f"expected two occupied worker slots, saw {two_worker_snapshot['occupied_slots']}")
        overlap = measured_overlap(useful_interval("t1", initial["t1"]), useful_interval("t2", initial["t2"]))
        if overlap["seconds"] <= 0:
            raise RuntimeError(f"useful worker execution did not overlap: {overlap}")

        initial_proofs = {}
        authority_timings = {}
        for ticket_id in ["t1", "t2"]:
            comparison, verification, acceptance, timings = self.verify_accept(
                initial[ticket_id]["candidate"],
                lead_auth,
                tickets_by_id[ticket_id],
                key_prefix=f"initial-{ticket_id}",
            )
            initial_proofs[ticket_id] = {
                "comparison": {"changed_paths": extract_diff_paths(comparison["diff"]), "contamination": []},
                "verification": verification,
                "acceptance": acceptance,
            }
            authority_timings[ticket_id] = timings

        land_started = time.time()
        first_landing = self.command(
            "land_candidate",
            {"candidate": initial["t1"]["candidate"]["candidate"], "expected_trunk": base},
            auth=lead_auth,
            key="land-initial-t1",
        )["result"]
        first_land_seconds = round(time.time() - land_started, 3)
        if first_landing["state"] != "landed":
            raise RuntimeError(f"first candidate did not land: {first_landing}")
        stale_started = time.time()
        stale_landing = self.command(
            "land_candidate",
            {"candidate": initial["t2"]["candidate"]["candidate"], "expected_trunk": base},
            auth=lead_auth,
            key="land-initial-t2-stale",
        )["result"]
        stale_land_seconds = round(time.time() - stale_started, 3)
        if stale_landing["state"] != "stale_base" or not stale_landing.get("requires_new_attempt"):
            raise RuntimeError(f"second candidate did not receive stale-base refusal: {stale_landing}")

        finish_started = time.time()
        initial_finish_snapshots = {
            "t1": self.finish_attempt(initial["t1"]["grant"]["attempt"]),
            "t2": self.finish_attempt(initial["t2"]["grant"]["attempt"]),
        }
        initial_finish_seconds = round(time.time() - finish_started, 3)
        if initial_finish_snapshots["t2"]["occupied_slots"] != 0:
            raise RuntimeError("initial worker attempts did not release owned slots before integration")

        integration_started = time.time()
        fresh = self.run_attempt(
            "t2",
            lead_auth,
            {
                "ticket": "t2",
                "files": ticket_overlay(tickets_by_id["t2"]),
                "check": PARTIAL_CONTRACTS["t1"]["check"] + PARTIAL_CONTRACTS["t2"]["check"],
            },
            key_prefix="fresh-t2",
        )
        fresh_attempt_seconds = round(time.time() - integration_started, 3)
        if fresh["grant"]["base"] != first_landing["commit"]:
            raise RuntimeError("fresh integration attempt did not start from current canonical base")

        old_proof_refusal = self.command(
            "accept_candidate",
            {
                "candidate": fresh["candidate"]["candidate"],
                "verification": initial_proofs["t2"]["verification"]["verification"],
            },
            auth=lead_auth,
            key="reject-old-proof-for-fresh-candidate",
            allow_error=True,
        )
        if "error" not in old_proof_refusal:
            raise RuntimeError("old proof unexpectedly authorized the fresh integration candidate")

        fresh_comparison, fresh_verification, fresh_acceptance, fresh_timings = self.verify_accept(
            fresh["candidate"],
            lead_auth,
            tickets_by_id["t2"],
            key_prefix="fresh-t2",
        )
        if fresh_verification["document"]["suite_id"] != f"{self.task.id}:complete":
            raise RuntimeError(f"fresh integration used unexpected verifier suite: {fresh_verification['document']['suite_id']}")
        final_land_started = time.time()
        final_landing = self.command(
            "land_candidate",
            {"candidate": fresh["candidate"]["candidate"], "expected_trunk": first_landing["commit"]},
            auth=lead_auth,
            key="land-fresh-t2",
        )["result"]
        final_land_seconds = round(time.time() - final_land_started, 3)
        if final_landing["state"] != "landed":
            raise RuntimeError(f"fresh integration candidate did not land: {final_landing}")
        fresh_finish_started = time.time()
        final_snapshot = self.finish_attempt(fresh["grant"]["attempt"])
        fresh_finish_seconds = round(time.time() - fresh_finish_started, 3)

        store = self.root / "authority/objects/store.git"
        final_ref = fleet.git(self.root / "canonical", "rev-parse", "refs/heads/protected").stdout.strip()
        final_external = independent_verify_commit(
            self.task, self.root / "canonical", final_ref, complete_suite, policy["verifier_runtime"], policy["tools"], self.root, "final"
        )
        t1_only = independent_verify_commit(
            self.task, store, initial["t1"]["candidate"]["commit"], complete_suite, policy["verifier_runtime"], policy["tools"], self.root, "t1-only"
        )
        t2_only = independent_verify_commit(
            self.task, store, initial["t2"]["candidate"]["commit"], complete_suite, policy["verifier_runtime"], policy["tools"], self.root, "t2-only"
        )
        tampered = independent_verify_commit(
            self.task, store, initial["t1"]["candidate"]["commit"], complete_suite, policy["verifier_runtime"], policy["tools"], self.root, "tampered-t1-only", tamper_tests=True
        )
        if final_external["verdict"] != "pass" or t1_only["verdict"] != "fail" or t2_only["verdict"] != "fail" or tampered["verdict"] != "fail":
            raise RuntimeError(
                "independent final verifier controls did not match expectations: "
                + json.dumps({"final": final_external, "t1_only": t1_only, "t2_only": t2_only, "tampered": tampered}, sort_keys=True)
            )

        inventory = self.query("workspace_inventory")
        events = self.query("events", {"after": 0, "limit": 200})
        flow_seconds = round(time.time() - flow_started, 3)
        cleanup_started = time.time()
        process_records = stopped_services(self.processes)
        cleanup_seconds = round(time.time() - cleanup_started, 3)
        attempt_records = [
            self.attempt_process_record(initial["t1"]["grant"]),
            self.attempt_process_record(initial["t2"]["grant"]),
            self.attempt_process_record(fresh["grant"]),
        ]
        useful_seconds = {
            ticket_id: round(
                (initial[ticket_id]["ready"]["useful_interval"]["end_monotonic_ns"]
                 - initial[ticket_id]["ready"]["useful_interval"]["start_monotonic_ns"]) / 1_000_000_000,
                6,
            )
            for ticket_id in ["t1", "t2"]
        }
        useful_seconds["fresh_t2"] = round(
            (fresh["ready"]["useful_interval"]["end_monotonic_ns"]
             - fresh["ready"]["useful_interval"]["start_monotonic_ns"]) / 1_000_000_000,
            6,
        )
        controller_verify_seconds = round(
            sum(authority_timings[t]["verify_seconds"] for t in ["t1", "t2"]) + fresh_timings["verify_seconds"],
            3,
        )
        independent_verify_seconds = round(
            sum(item["seconds"] for item in [final_external, t1_only, t2_only, tampered]),
            3,
        )
        timing = {
            "wall_seconds": round(time.time() - started, 3),
            "setup_seconds": setup_seconds,
            "controller_flow_seconds": flow_seconds,
            "attempt_seconds": round(sum(useful_seconds.values()), 3),
            "initial_attempt_wall_seconds": initial_attempt_wall_seconds,
            "useful_attempt_seconds": useful_seconds,
            "verify_seconds": round(controller_verify_seconds + independent_verify_seconds, 3),
            "controller_verify_seconds": controller_verify_seconds,
            "independent_verify_seconds": independent_verify_seconds,
            "accept_seconds": round(sum(authority_timings[t]["accept_seconds"] for t in ["t1", "t2"]) + fresh_timings["accept_seconds"], 3),
            "land_seconds": round(first_land_seconds + stale_land_seconds + final_land_seconds, 3),
            "initial_finish_seconds": initial_finish_seconds,
            "fresh_attempt_wall_seconds": fresh_attempt_seconds,
            "fresh_finish_seconds": fresh_finish_seconds,
            "cleanup_seconds": cleanup_seconds,
        }
        complete_contract = {
            "suite_id": fresh_verification["document"]["suite_id"],
            "suite_digest": fresh_verification["document"]["suite"],
            "selected_tests": complete_suite["selected_tests"],
            "protected_test_sha256": complete_suite["protected_test_sha256"],
            "complete_original_test_file_restored": True,
            "controller_verdict": fresh_verification["verdict"],
            "final_verdict": final_external["verdict"],
            "final_external": final_external,
            "omitted_controls": {"t1_only": t1_only["verdict"], "t2_only": t2_only["verdict"]},
            "omitted_control_evidence": {"t1_only": t1_only, "t2_only": t2_only},
            "tampered_tests_control": tampered["verdict"],
            "tampered_tests_evidence": tampered,
        }
        partial_contracts = {
            ticket_id: {
                "suite_id": initial_proofs[ticket_id]["verification"]["document"]["suite_id"],
                "suite_digest": initial_proofs[ticket_id]["verification"]["document"]["suite"],
                "selected_tests": PARTIAL_CONTRACTS[ticket_id]["selected_tests"],
                "protected_test_sha256": partial_suite["protected_test_sha256"],
                "fixed_verdict": initial_proofs[ticket_id]["verification"]["verdict"],
                "candidate": initial[ticket_id]["candidate"],
                "acceptance": initial_proofs[ticket_id]["acceptance"],
            }
            for ticket_id in ["t1", "t2"]
        }
        orchestration_passed = (
            all(partial_contracts[t]["fixed_verdict"] == "pass" for t in ["t1", "t2"])
            and first_landing["state"] == "landed"
            and stale_landing["state"] == "stale_base"
            and "error" in old_proof_refusal
            and fresh_verification["verdict"] == "pass"
            and fresh_acceptance["candidate"] == fresh["candidate"]["candidate"]
            and final_landing["state"] == "landed"
            and final_ref == fresh["candidate"]["commit"]
            and complete_contract["final_verdict"] == "pass"
            and complete_contract["omitted_controls"] == {"t1_only": "fail", "t2_only": "fail"}
            and complete_contract["tampered_tests_control"] == "fail"
            and overlap["seconds"] > 0
            and any(a["id"] == fresh["grant"]["attempt"] and a["state"] == "exited" for a in final_snapshot["attempts"])
        )
        return {
            "schema_version": CONTROLLER_SCHEMA,
            "mode": "deterministic",
            "task": self.task.id,
            "ticket": "complete",
            "run_root": str(self.root),
            "source": self.source,
            "build": self.build,
            "control": control,
            "roles": {
                "lead": control["lead"],
                "initial_attempts": {ticket_id: public_grant(initial[ticket_id]["grant"]) for ticket_id in ["t1", "t2"]},
                "fresh_integration_attempt": public_grant(fresh["grant"]),
            },
            "kata": {"binary": kata["binary"], "instance_uid": kata["instance_uid"], "project": kata["project"],
                     "issues": issues, "config_sha256": policy["kata_config_sha256"]},
            "git": {"policy_sha256": policy["git_policy_sha256"], "canonical": str(self.root / "canonical"),
                    "protected_ref": "refs/heads/protected", "base": base, "after_first_landing": first_landing["commit"], "final": final_ref},
            "supervisor": {"config_sha256": policy["supervisor_config_sha256"]},
            "runtime_closure": {
                "status": "pinned",
                "pinned": {
                    "partial_verification_adapter": {"path": partial_suite["path"], "sha256": partial_suite["sha256"]},
                    "complete_verification_adapter": {"path": complete_suite["path"], "sha256": complete_suite["sha256"]},
                    "protected_test_sha256": complete_suite["protected_test_sha256"],
                    "git": policy["tools"]["git"],
                    "python_executable": policy["tools"]["python"],
                    "bubblewrap": policy["tools"]["bubblewrap"],
                    "verifier_runtime": policy["verifier_runtime"],
                },
                "enforced": "sqlite-controller validates the pinned runtime manifest, ro-binds that private snapshot at /usr, and binds the runtime digest into verifier proof",
            },
            "partial_contract": {
                "suite_id": "multiple",
                "suite_digest": "multiple",
                "selected_tests": complete_suite["selected_tests"],
                "noop_verdict": "not_run",
                "fixed_verdict": "pass" if all(partial_contracts[t]["fixed_verdict"] == "pass" for t in ["t1", "t2"]) else "fail",
            },
            "partial_contracts": partial_contracts,
            "complete_contract": complete_contract,
            "initial_candidates": {ticket_id: initial[ticket_id]["candidate"] for ticket_id in ["t1", "t2"]},
            "comparison": {
                "changed_paths": sorted(set(initial_proofs["t1"]["comparison"]["changed_paths"] + fresh["ready"]["changed"])),
                "contamination": [],
            },
            "landing_sequence": [
                {"ticket": "t1", **first_landing},
                {"ticket": "t2", **final_landing},
            ],
            "landing": final_landing,
            "stale_landing": stale_landing,
            "fresh_integration": {
                "attempt_count": 1,
                "base": fresh["grant"]["base"],
                "candidate": fresh["candidate"],
                "comparison": {"changed_paths": extract_diff_paths(fresh_comparison["diff"]), "contamination": []},
                "old_proof_for_fresh_candidate": "rejected",
                "old_proof_for_fresh_candidate_error": old_proof_refusal["error"],
                "verification": fresh_verification,
                "acceptance": fresh_acceptance,
                "landing": final_landing,
            },
            "useful_execution_overlap": overlap,
            "capacity": {
                "initial_occupied_slots": two_worker_snapshot["occupied_slots"],
                "after_initial_release_occupied_slots": initial_finish_snapshots["t2"]["occupied_slots"],
                "max_concurrent_worker_executions": 2,
            },
            "process_disposition": {"owned_attempts": "stopped", "services": "stopped", "processes": process_records,
                                    "attempts": attempt_records},
            "events_observed": {"count": len(events), "last_seq": events[-1]["seq"] if events else 0},
            "workspace_inventory": {
                "configured": inventory.get("configured"),
                "backend": inventory.get("backend"),
                "workspaces": len(inventory.get("workspaces", [])),
                "history": len(inventory.get("history", [])),
            },
            "timing": timing,
            "qualification_limits": [
                "deterministic m01 complete path only",
                "no live model calls",
                "no dependent-task, textual-conflict or broad recovery qualification",
                "no permanent readiness claim",
            ],
            "orchestration_passed": orchestration_passed,
        }


def controller_ticket(task: tasklib.Task, ticket_id: str) -> fleet.Ticket:
    if task.id != "m01-disjoint-utils":
        raise SystemExit("controller mode currently supports only m01-disjoint-utils")
    tickets = {ticket.id: ticket for ticket in fleet.load_tickets(task)}
    if ticket_id not in tickets or ticket_id not in PARTIAL_CONTRACTS:
        raise SystemExit(f"controller mode unsupported ticket {ticket_id!r}")
    ticket = tickets[ticket_id]
    if ticket.depends:
        raise SystemExit("controller mode rejects dependency graphs before execution")
    if set(ticket.allow) != set(PARTIAL_CONTRACTS[ticket_id]["allow"]):
        raise SystemExit("controller mode task allowlist does not match the pinned partial contract")
    return ticket


def controller_tickets(task: tasklib.Task, ticket_id: str) -> list[fleet.Ticket]:
    if ticket_id != "complete":
        return [controller_ticket(task, ticket_id)]
    if task.id != "m01-disjoint-utils":
        raise SystemExit("controller mode currently supports only m01-disjoint-utils")
    tickets = {ticket.id: ticket for ticket in fleet.load_tickets(task)}
    if set(tickets) != {"t1", "t2"}:
        raise SystemExit("controller complete mode requires m01 tickets t1 and t2")
    selected = [tickets["t1"], tickets["t2"]]
    for ticket in selected:
        if ticket.depends:
            raise SystemExit("controller mode rejects dependency graphs before execution")
        if set(ticket.allow) != set(PARTIAL_CONTRACTS[ticket.id]["allow"]):
            raise SystemExit("controller mode task allowlist does not match the pinned partial contract")
    return selected


def validate_controller_selection(tasks: list[tasklib.Task], ticket_id: str) -> None:
    for task in tasks:
        controller_tickets(task, ticket_id)


def run_controller_condition(task: tasklib.Task, agent, out: Path, index: int, ticket_id: str, rid: str | None = None,
                             observer=None) -> dict:
    if agent.kind != "reference":
        raise SystemExit("controller mode currently supports the deterministic reference agent only")
    observer = observer or (lambda event, **fields: None)
    tickets = controller_tickets(task, ticket_id)
    ticket = tickets[0]
    complete = ticket_id == "complete"
    verify_record = tasklib.capsule_for(task)
    rid = rid or f"{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}-controller{ticket_id}-{task.id}-reference-{index:03d}"
    total_started = time.time()
    root = Path(tempfile.mkdtemp(prefix=f"cb-{ticket_id}-", dir=str(run_root_parent())))
    source_started = time.time()
    source = source_identity()
    source_capture_seconds = round(time.time() - source_started, 3)
    write_json(root / "source-manifest.json", source["content_manifest"])
    observer("controller-setup")
    build = build_controller(root)
    run = ControllerRun(task, ticket, root, build, source)
    try:
        controller = run.run_complete(tickets) if complete else run.run()
    except BaseException as exc:
        process_records = stopped_services(run.processes)
        failure = {
            "schema_version": CONTROLLER_SCHEMA,
            "mode": "deterministic",
            "task": task.id,
            "ticket": ticket_id,
            "run_root": str(root),
            "source": source,
            "build": build,
            "failure_stage": "controller-run",
            "error": f"{type(exc).__name__}: {exc}",
            "process_disposition": {"services": "stopped", "processes": process_records},
        }
        write_json(root / "failure.json", failure)
        raise
    observer("controller-done", passed=controller["orchestration_passed"])
    total_wall_seconds = round(time.time() - total_started, 3)
    task_passed = (
        controller.get("complete_contract", {}).get("final_verdict") == "pass"
        if complete else controller["partial_contract"]["fixed_verdict"] == "pass"
    )
    rec = {
        "schema_version": "corenius.bench.result.v1",
        "run_id": rid,
        "task_id": task.id,
        "condition": f"controller:{ticket_id}",
        "agent": {"spec": agent.spec},
        "capsule": {
            "digest": verify_record["capsule_digest"],
            "net": verify_record["policy"]["net"],
            "hermetic": verify_record["hermetic"],
            "non_hermetic_layers": verify_record["non_hermetic_layers"],
        },
        "outcome": {
            "passed": bool(controller["orchestration_passed"]),
            "reward": 1.0 if controller["orchestration_passed"] else 0.0,
            "verifier_rc": 0 if task_passed else 1,
            "timed_out": False,
            "agent_rc": 0,
            "task_verdict": "pass" if complete and task_passed else ("partial_pass" if task_passed else "fail"),
            "orchestration_verdict": "pass" if controller["orchestration_passed"] else "fail",
            "effect_disposition": "resolved",
        },
        "timing": {
            "wall_seconds": total_wall_seconds,
            "agent_seconds": controller["timing"]["attempt_seconds"],
            "verify_seconds": controller["timing"]["verify_seconds"],
            "source_capture_seconds": source_capture_seconds,
            "build_seconds": controller["build"]["seconds"],
            "setup_seconds": round(source_capture_seconds + controller["build"]["seconds"] + controller["timing"]["setup_seconds"], 3),
            "controller_flow_seconds": controller["timing"]["controller_flow_seconds"],
            "cleanup_seconds": controller["timing"]["cleanup_seconds"],
        },
        "coordination": {
            "conflicts": 0,
            "redos": 1 if complete else 0,
            "stale_refusals": 1 if complete and controller["stale_landing"]["state"] == "stale_base" else 0,
            "contamination": 0,
            "contamination_paths": [],
            "tickets": 2 if complete else 1,
            "tickets_landed": 2 if complete and controller["fresh_integration"]["landing"]["state"] == "landed" else (
                1 if not complete and controller["landing"]["state"] == "landed" else 0
            ),
            "condition_mode": "controller",
            "attempts": 3 if complete else 2,
            "manual_interventions": 0,
        },
        "cost": {"usd": None, "input_tokens": None, "output_tokens": None},
        "provenance": {
            "outcome": "controller measured",
            "timing": "measured by harness phase timers",
            "coordination": "controller measured",
            "cost": "unmeasured; deterministic no-model mode",
        },
        "controller": controller,
    }
    return rec
