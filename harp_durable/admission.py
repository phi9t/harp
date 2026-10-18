"""Freeze Git-managed intent and source before submitting durable coding work."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tempfile

from controller.durable.common import atomic_json, digest, fsync_dir, read_json, verify_input_bundle

WORKFLOW_VERSION = "coding-v1"


def relative_path(value):
    path = PurePosixPath(value)
    if (not isinstance(value, str) or not value or path.is_absolute() or
            str(path) != value or any(p in {"..", ".git"} for p in path.parts) or
            "\\" in value or "\x00" in value):
        raise ValueError("expected a canonical repository-relative path outside .git")
    return value


def git(repo, *args, output=None):
    environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL="/dev/null", GIT_TERMINAL_PROMPT="0")
    result = subprocess.run(["git", "--no-optional-locks", "-c", "core.hooksPath=/dev/null",
                           "-c", "core.fsmonitor=false", "-C", str(repo), *args],
                          env=environment, stdout=output or subprocess.PIPE,
                          stderr=subprocess.PIPE)
    if result.returncode:
        raise ValueError("Git admission failed: " + result.stderr.decode(errors="replace")[:4096].strip())
    return result.stdout


def export_bundle(repo, base, destination, temporary):
    # A raw commit ID is not an advertised bundle ref. Create that ref only in
    # an owned export repository, leaving the source checkout and its refs alone.
    export = Path(temporary) / "export.git"
    export.mkdir()
    git(export, "init", "--bare", "-q")
    git(export, "fetch", "--no-tags", str(repo), base + ":refs/heads/source")
    git(export, "bundle", "create", str(destination), "refs/heads/source")


def commit(repo, ref):
    result = git(repo, "rev-parse", "--verify", "--end-of-options", ref + "^{commit}").decode().strip()
    if not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", result):
        raise ValueError("unsupported Git object identity")
    return result


def blob(repo, revision, path):
    relative_path(path)
    entry = git(repo, "ls-tree", "-z", revision, "--", path)
    if not entry or entry.split(b" ", 1)[0] not in {b"100644", b"100755"}:
        raise ValueError("task and acceptance inputs must be committed regular files")
    data = git(repo, "cat-file", "blob", revision + ":" + path)
    if len(data) > 256 * 1024:
        raise ValueError("task/acceptance input exceeds 256 KiB")
    return data


def acceptance(data):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("duplicate acceptance field")
            result[key] = value
        return result
    value = json.loads(data, object_pairs_hook=unique)
    if not isinstance(value, dict) or set(value) != {"checks"}:
        raise ValueError("acceptance requires exactly a checks array")
    if not isinstance(value["checks"], list) or not 1 <= len(value["checks"]) <= 32:
        raise ValueError("acceptance requires one through 32 checks")
    for argv in value["checks"]:
        if (not isinstance(argv, list) or not argv or
                any(not isinstance(a, str) or not a or "\x00" in a for a in argv)):
            raise ValueError("acceptance checks must be nonempty argv arrays")
    return value


def submit(store, execution, *, run_id, repository, task, acceptance_path,
           base="HEAD", task_revision="HEAD", provider="deterministic", model=None,
           fixture=None):
    if execution is None or execution.backend != "cgroup":
        raise ValueError("coding admission requires an admitted Insula cgroup execution")
    execution.verify()
    inputs_root = Path(execution.config.get("inputs_root", ""))
    if inputs_root != store.root / "inputs":
        raise ValueError("coding inputs must belong to this project state")
    if provider not in {"deterministic", "codex", "trae"}:
        raise ValueError("unsupported provider")
    repo = Path(repository).resolve(strict=True)
    top = Path(git(repo, "rev-parse", "--show-toplevel").decode().strip()).resolve()
    if top != repo:
        raise ValueError("repository must name its Git working-tree root")
    base_id, task_id = commit(repo, base), commit(repo, task_revision)
    task_bytes = blob(repo, task_id, task)
    task_bytes.decode("utf-8")
    acceptance_bytes = blob(repo, task_id, acceptance_path)
    acceptance(acceptance_bytes)
    if fixture is not None:
        if provider != "deterministic" or set(fixture) != {"path", "content", "sleep"}:
            raise ValueError("fixture edits are exclusive to deterministic activities")
        relative_path(fixture["path"])
        if not isinstance(fixture["content"], str) or len(fixture["content"].encode()) > 256 * 1024:
            raise ValueError("invalid fixture content")
        if type(fixture["sleep"]) not in {int, float} or not 0 <= fixture["sleep"] <= 60:
            raise ValueError("invalid fixture delay")
    request = {"schema": 1, "run_id": run_id, "source_repository": str(repo),
               "task_revision": task_id, "task_path": task, "base": base_id,
               "task_sha256": hashlib.sha256(task_bytes).hexdigest(),
               "acceptance_path": acceptance_path,
               "acceptance_sha256": hashlib.sha256(acceptance_bytes).hexdigest(),
               "provider": provider, "model": model, "fixture": fixture,
               "workflow_version": WORKFLOW_VERSION, "execution_contract": execution.contract}
    identity = digest(request)
    old = store.db.execute("SELECT args FROM runs WHERE id=?", (run_id,)).fetchone()
    if old:
        admitted = json.loads(old[0])
        if admitted.get("identity") != identity:
            raise ValueError("run ID already binds different immutable inputs")
        verify_input_bundle(inputs_root, admitted["inputs"])
        return {"run_id": run_id, **admitted, "reused": True}
    with tempfile.TemporaryDirectory(dir=inputs_root, prefix=".admitting-") as staging:
        bundle = Path(staging) / "bundle"
        bundle.mkdir(mode=0o700)
        (bundle / "task.md").write_bytes(task_bytes)
        (bundle / "acceptance.json").write_bytes(acceptance_bytes)
        atomic_json(bundle / "request.json", request)
        export_bundle(repo, base_id, bundle / "source.bundle", staging)
        files = {}
        for path in sorted(bundle.iterdir()):
            with path.open("rb") as stream:
                sha = hashlib.file_digest(stream, "sha256").hexdigest()
                os.fsync(stream.fileno())
            files[path.name] = {"sha256": sha, "size": path.stat().st_size}
            path.chmod(0o400)
        manifest = {"schema": 1, "files": files}
        bundle_id = digest(manifest)
        atomic_json(bundle / "manifest.json", manifest)
        (bundle / "manifest.json").chmod(0o400)
        fsync_dir(bundle)
        destination = inputs_root / bundle_id
        # Rename only a fully durable directory; concurrent identical submissions
        # converge on one verified input artifact and Store's immutable run ID.
        try:
            bundle.rename(destination)
        except OSError:
            if not destination.exists():
                raise
            verify_input_bundle(inputs_root, bundle_id)
        fsync_dir(inputs_root)
        verify_input_bundle(inputs_root, bundle_id)
    args = {"identity": identity, "inputs": bundle_id, "provider": provider}
    store.submit(run_id, "coding", WORKFLOW_VERSION, args, execution.contract)
    return {"run_id": run_id, **args, "reused": False}
