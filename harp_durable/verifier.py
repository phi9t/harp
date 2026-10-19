"""Independent exact-tree verification; acceptance checks cannot write its proof."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import selectors
import stat
import subprocess
import sys

from controller.durable.common import atomic_json, digest, publish_result, read_json
from .admission import acceptance, git
from .candidates import check_git_subject

MAX_LOG_BYTES = 1 << 20


def tracked_tree(workspace, commit):
    result = []
    for entry in git(workspace, "ls-tree", "-r", "-z", commit).split(b"\0"):
        if not entry:
            continue
        metadata, raw_name = entry.split(b"\t", 1)
        mode, kind, blob = metadata.decode().split()
        if kind != "blob" or mode not in {"100644", "100755", "120000"}:
            raise ValueError("verification does not support Git submodules or special tree entries")
        name = os.fsdecode(raw_name)
        relative = Path(name)
        if relative.is_absolute() or any(p in {"..", ".git"} for p in relative.parts):
            raise ValueError("unsafe tracked candidate path")
        result.append((name, mode, blob))
    return result


def unchanged(workspace, tree):
    """Ignore the mutable index and compare filesystem bytes directly to Git blobs."""
    try:
        for name, mode, expected in tree:
            path = workspace / name
            if any(parent.is_symlink() for parent in path.parents if parent != workspace.parent):
                return False
            info = path.lstat()
            hashed = hashlib.sha1() if len(expected) == 40 else hashlib.sha256()
            if mode == "120000":
                if not stat.S_ISLNK(info.st_mode):
                    return False
                data = os.fsencode(os.readlink(path))
                hashed.update(b"blob " + str(len(data)).encode() + b"\0" + data)
            else:
                if not stat.S_ISREG(info.st_mode) or bool(info.st_mode & stat.S_IXUSR) != (mode == "100755"):
                    return False
                fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
                with os.fdopen(fd, "rb") as source:
                    actual = os.fstat(source.fileno())
                    if not stat.S_ISREG(actual.st_mode):
                        return False
                    hashed.update(b"blob " + str(actual.st_size).encode() + b"\0")
                    for block in iter(lambda: source.read(1 << 20), b""):
                        hashed.update(block)
            if hashed.hexdigest() != expected:
                return False
    except OSError:
        return False
    return True


def sandbox_argv(workspace, argv):
    """A check sees source and private scratch, never the parent's writable proof."""
    child = "/home/bench/run/workspace"
    command = ["/usr/bin/bwrap", "--die-with-parent", "--new-session", "--unshare-user",
               "--unshare-pid", "--unshare-ipc", "--unshare-uts", "--unshare-net",
               "--unshare-cgroup-try", "--cap-drop", "ALL", "--ro-bind", "/", "/",
               "--tmpfs", "/home/bench", "--dir", "/home/bench/run",
               "--proc", "/proc", "--dev", "/dev", "--tmpfs", "/run",
               "--dir", "/run/corenius-tmp", "--dir", "/home/bench/run/home",
               "--bind", str(workspace), child,
               "--ro-bind", str(workspace / ".git"), child + "/.git", "--clearenv"]
    environment = {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8",
                   "HOME": "/home/bench/run/home", "TMPDIR": "/run/corenius-tmp",
                   "TMP": "/run/corenius-tmp", "TEMP": "/run/corenius-tmp",
                   "PYTHONNOUSERSITE": "1", "PYTHONDONTWRITEBYTECODE": "1",
                   "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_CONFIG_NOSYSTEM": "1",
                   "GIT_TERMINAL_PROMPT": "0", "GIT_OPTIONAL_LOCKS": "0"}
    for name, value in environment.items():
        command += ["--setenv", name, value]
    return command + ["--chdir", child, "--", *argv]


def run_check(command, logs, index):
    """Drain pipes with bounded retained logs; the activity guardian owns deadlines.

    The nested PID namespace ends with bwrap, so descendant processes cannot keep
    mutating source after a check returns. A wedged check never yields a proof:
    the outer guardian times out and cleans its complete delegated cgroup.
    """
    paths = {stream: logs / f"{index:03d}.{stream}.log" for stream in ("stdout", "stderr")}
    sizes = {stream: 0 for stream in paths}
    with paths["stdout"].open("xb", buffering=0) as out, paths["stderr"].open("xb", buffering=0) as err:
        handles = {"stdout": out, "stderr": err}
        with subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, close_fds=True, env={"PATH": "/usr/bin:/bin"}) as process:
            with selectors.DefaultSelector() as selector:
                selector.register(process.stdout, selectors.EVENT_READ, "stdout")
                selector.register(process.stderr, selectors.EVENT_READ, "stderr")
                while selector.get_map():
                    for key, _ in selector.select():
                        data = os.read(key.fileobj.fileno(), 65536)
                        if not data:
                            selector.unregister(key.fileobj)
                            continue
                        name = key.data
                        remaining = max(0, MAX_LOG_BYTES - sizes[name])
                        handles[name].write(data[:remaining])
                        sizes[name] += len(data)
            code = process.wait()
        for stream in handles.values():
            stream.flush()
            os.fsync(stream.fileno())
    return {"returncode": code, "logs": {
        name: {"path": str(path.relative_to(logs.parent)),
               "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
               "retained_bytes": min(sizes[name], MAX_LOG_BYTES), "output_bytes": sizes[name],
               "truncated": sizes[name] > MAX_LOG_BYTES}
        for name, path in paths.items()}}


def verify(envelope, inputs, attempt):
    record = read_json(inputs / "verification.json")
    args = envelope["input"]
    subject = record["subject"]
    if (record.get("schema") != 1 or digest(subject) != record["subject_digest"]
            or record["subject_digest"] != args["subject_digest"]
            or os.environ.get("CORENIUS_DURABLE_CONTRACT") != args["execution_contract"]):
        raise ValueError("verification subject or execution binding differs")
    request = read_json(inputs / "request.json")
    task = (inputs / "task.md").read_bytes()
    check_bytes = (inputs / "acceptance.json").read_bytes()
    checks = acceptance(check_bytes)
    if (digest(request) != subject["identity"] or request["base"] != subject["base"]
            or hashlib.sha256(task).hexdigest() != request["task_sha256"]
            or hashlib.sha256(check_bytes).hexdigest() != request["acceptance_sha256"]
            or digest(checks) != record["checks_digest"] or digest(checks) != args["checks_digest"]):
        raise ValueError("verification checks or admitted inputs differ")
    with (inputs / "candidate.bundle").open("rb") as stream:
        if hashlib.file_digest(stream, "sha256").hexdigest() != subject["bundle_sha256"]:
            raise ValueError("captured candidate bundle changed")
    workspace = attempt / "workspace"
    check_git_subject(inputs / "candidate.bundle", subject, workspace)
    tree = tracked_tree(workspace, subject["commit"])
    if not unchanged(workspace, tree):
        raise ValueError("independent checkout differs from captured Git tree")
    logs = attempt / "checks"
    logs.mkdir(mode=0o700)
    proof = {"schema": 1, "subject": subject, "subject_digest": args["subject_digest"],
             "inputs": args["inputs"], "checks_digest": args["checks_digest"],
             "execution_contract": args["execution_contract"],
             "verification_attempt": envelope["attempt_id"], "passed": False, "checks": []}
    for index, argv in enumerate(checks["checks"]):
        atomic_json(attempt / "progress.json", {**proof, "running_check": index, "argv": argv})
        outcome = run_check(sandbox_argv(workspace, argv), logs, index)
        outcome.update(argv=argv, tracked_source_unchanged=unchanged(workspace, tree))
        proof["checks"].append(outcome)
        atomic_json(attempt / "progress.json", proof)
        if not outcome["tracked_source_unchanged"]:
            break
    proof["passed"] = (len(proof["checks"]) == len(checks["checks"])
                       and all(c["returncode"] == 0 and c["tracked_source_unchanged"] for c in proof["checks"]))
    return proof


def main():
    envelope = json.load(sys.stdin)
    proof = verify(envelope, Path("/home/bench/input"), Path(envelope["attempt_dir"]))
    publish_result(envelope, proof)


if __name__ == "__main__":
    main()
