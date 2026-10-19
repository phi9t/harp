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

from durable.common import atomic_json, digest, fsync_dir, read_json, verify_input_bundle

WORKFLOW_VERSION = "coding-v3"


from durable.estate.git import blob, commit, export_bundle, git, relative_path
from durable.intent.policy import acceptance


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
    provider_binding = None
    if provider != "deterministic":
        from durable.jobs.coding.provider_preflight import verify_providers
        if "providers" not in execution.config:
            raise ValueError("provider versions and capabilities must be qualified during preparation")
        verify_providers(execution.config["capsule"], execution.config["providers"])
        if provider not in execution.config["providers"]:
            raise ValueError("provider is not admitted by this execution profile")
        provider_binding = execution.config["providers"][provider]
        layers = execution.config["capsule"]["layers"]
        if (execution.config["capsule"]["policy"]["net"] != "host" or
                not any(binding["dest"] == "/cap/providers/" + provider
                        for layer in layers for binding in layer.get("file_binds", [])) or
                not any(binding["dest"] == f"/home/bench/credentials/{provider}-auth.json"
                        for layer in layers for binding in layer.get("binds", []))):
            raise ValueError("provider binary, credential and networking must be explicitly admitted")
    if model is not None and (not isinstance(model, str) or not model.strip() or "\x00" in model):
        raise ValueError("model must be a nonempty configured string")
    repo = Path(repository).resolve(strict=True)
    top = Path(git(repo, "rev-parse", "--show-toplevel").decode().strip()).resolve()
    if top != repo:
        raise ValueError("repository must name its Git working-tree root")
    from durable.estate.workspaces import Estate
    repository_id = Estate(store).register(repo)
    from durable.authority import protect_owner
    protect_owner(store.root, [repo], db=store.db)
    base_id, task_id = commit(repo, base), commit(repo, task_revision)
    task_bytes = blob(repo, task_id, task)
    task_bytes.decode("utf-8")
    acceptance_bytes = blob(repo, task_id, acceptance_path)
    policy = acceptance(acceptance_bytes)
    from durable.intent.records import freeze_records
    from durable.intent.assignment import bind
    records = freeze_records(task, task_bytes, lambda path: blob(repo, task_id, path))
    assignment = bind(records, policy)
    from durable.intent.inputs import freeze
    oracle_inputs = freeze(store.root / "oracles", {name: blob(repo, task_id, name)
                           for name in policy.get("oracle_files", [])})
    if fixture is not None:
        if provider != "deterministic" or set(fixture) != {"path", "content", "sleep"}:
            raise ValueError("fixture edits are exclusive to deterministic activities")
        relative_path(fixture["path"])
        if not isinstance(fixture["content"], str) or len(fixture["content"].encode()) > 256 * 1024:
            raise ValueError("invalid fixture content")
        if type(fixture["sleep"]) not in {int, float} or not 0 <= fixture["sleep"] <= 60:
            raise ValueError("invalid fixture delay")
    request = {"schema": 1, "run_id": run_id, "source_repository": str(repo), "repository_id": repository_id,
               "task_revision": task_id, "task_path": task, "base": base_id,
               "task_sha256": hashlib.sha256(task_bytes).hexdigest(),
               "assignment": assignment,
               "acceptance_path": acceptance_path,
               "acceptance_sha256": hashlib.sha256(acceptance_bytes).hexdigest(),
               "oracle_inputs": oracle_inputs,
               "provider": provider, "model": model, "fixture": fixture,
               "workflow_version": WORKFLOW_VERSION, "execution_contract": execution.contract}
    if provider_binding is not None:
        request["provider_binding"] = provider_binding
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
        atomic_json(bundle / "intent.json", {name: data.decode("utf-8")
                                           for name, data in records.items()})
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
    args = {"identity": identity, "inputs": bundle_id, "provider": provider,
            "workspace": {"repository": repository_id, "base": base_id}}
    if provider_binding is not None:
        args["provider_binding"] = provider_binding
    store.submit(run_id, "coding", WORKFLOW_VERSION, args, execution.contract)
    return {"run_id": run_id, **args, "reused": False}
