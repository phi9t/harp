"""Capture proposals and submit exact-subject verification to the durable authority."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import stat
import tempfile

from durable.common import (atomic_json, digest, fsync_dir,
                                       read_json, verify_input_bundle, verify_inputs_root)
from durable.agents.admission import acceptance, git

VERSION = "verification-v2"
MAX_BUNDLE_BYTES = 512 * 1024 * 1024


def ordinary(path):
    path = Path(path)
    if any(parent.is_symlink() for parent in (path, *path.parents)):
        raise ValueError("candidate paths must not contain symlinks")
    return path


def copy_file(source, destination, expected=None, limit=MAX_BUNDLE_BYTES):
    """Copy bounded regular bytes, rejecting links and concurrent source changes."""
    ordinary(source)
    fd = os.open(source, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as inp, open(destination, "xb") as out:
        before = os.fstat(inp.fileno())
        if not stat.S_ISREG(before.st_mode) or before.st_size > limit:
            raise ValueError("candidate input must be a bounded regular file")
        hashed, size = hashlib.sha256(), 0
        while block := inp.read(1 << 20):
            size += len(block)
            if size > limit:
                raise ValueError("candidate input exceeds its byte limit")
            hashed.update(block)
            out.write(block)
        after = os.fstat(inp.fileno())
        fields = lambda info: (info.st_dev, info.st_ino, info.st_mode, info.st_size,
                               info.st_mtime_ns, info.st_ctime_ns)
        if fields(before) != fields(after) or size != before.st_size:
            raise ValueError("candidate input changed while being captured")
        sha = hashed.hexdigest()
        if expected is not None and sha != expected:
            raise ValueError("candidate input digest disagrees with accepted evidence")
        out.flush()
        os.fsync(out.fileno())
    return {"sha256": sha, "size": size}


def check_git_subject(bundle, subject, destination, *, bare=False):
    """Import only committed objects; never read a worker's mutable Git directory."""
    heads = git(destination.parent, "bundle", "list-heads", str(bundle)).decode().splitlines()
    if heads != [subject["commit"] + " HEAD"]:
        raise ValueError("candidate bundle does not advertise the captured commit")
    git(destination.parent, "clone", "--bare" if bare else "--no-checkout", "--",
        str(bundle), str(destination))
    actual = git(destination, "rev-parse", "--verify", "HEAD^{commit}").decode().strip()
    tree = git(destination, "rev-parse", "--verify", "HEAD^{tree}").decode().strip()
    if (actual, tree) != (subject["commit"], subject["tree"]):
        raise ValueError("candidate commit/tree differs from the captured subject")
    git(destination, "merge-base", "--is-ancestor", subject["base"], actual)
    if not bare:
        git(destination, "checkout", "--detach", actual)


def _proposal(store, run_id):
    run = store.status(run_id)
    if run["workflow"] != "coding" or run["status"] != "SUCCEEDED":
        raise ValueError("capture requires a completed guardian-accepted coding proposal")
    result = run["result"]
    required = {"schema", "identity", "attempt_id", "base", "provider", "commit", "tree",
                "bundle", "bundle_sha256"}
    if (not isinstance(result, dict) or not required <= set(result)
            or set(result) - required - {"provider_report"}
            or type(result["schema"]) is not int or result["schema"] != 1):
        raise ValueError("invalid coding proposal")
    for name in ("base", "commit", "tree"):
        if not isinstance(result[name], str) or not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", result[name]):
            raise ValueError("candidate requires exact Git object identities")
    if (result["bundle"] != "candidate.bundle" or not isinstance(result["bundle_sha256"], str)
            or not re.fullmatch(r"[0-9a-f]{64}", result["bundle_sha256"])
            or not isinstance(result["attempt_id"], str)
            or not re.fullmatch(r"[0-9a-f]{32}", result["attempt_id"])):
        raise ValueError("invalid candidate bundle identity")
    attempts = [a for a in run["attempts"] if a["id"] == result["attempt_id"]]
    if len(attempts) != 1:
        raise ValueError("proposal has no accepted coding attempt")
    attempt = attempts[0]
    operations = [o for o in run["operations"] if o["id"] == attempt["operation_id"]]
    if (attempt["status"] != "SUCCEEDED" or not attempt["receipt_digest"] or len(operations) != 1
            or operations[0]["kind"] != "ACTIVITY" or operations[0]["key"] != "coding"
            or operations[0]["status"] != "SUCCEEDED"
            or operations[0]["active_attempt"] is not None
            or operations[0]["attempt_count"] != attempt["number"]
            or json.loads(operations[0]["result"]) != result):
        raise ValueError("proposal is not the current committed coding outcome")
    inputs = verify_input_bundle(store.root / "inputs", run["args"]["inputs"])
    request = read_json(inputs / "request.json")
    if (digest(request) != run["args"]["identity"] or result["identity"] != run["args"]["identity"]
            or request["base"] != result["base"] or request["provider"] != result["provider"]
            or request["run_id"] != run_id or request["execution_contract"] != run["contract"]
            or request["workflow_version"] != run["version"]):
        raise ValueError("candidate differs from admitted task or execution identity")
    for name, key in (("task.md", "task_sha256"), ("acceptance.json", "acceptance_sha256")):
        if hashlib.sha256((inputs / name).read_bytes()).hexdigest() != request[key]:
            raise ValueError("admitted task or acceptance bytes changed")
    acceptance((inputs / "acceptance.json").read_bytes())
    subject = {"schema": 1, "coding_run": run_id, "identity": result["identity"],
               "coding_inputs": run["args"]["inputs"], "base": result["base"],
               "commit": result["commit"], "tree": result["tree"],
               "coding_attempt": attempt["id"], "provider": result["provider"],
               "workflow_version": run["version"],
               "execution_contract": run["contract"],
               "receipt_digest": attempt["receipt_digest"],
               "bundle_sha256": result["bundle_sha256"]}
    return subject, attempt, inputs


def capture(store, run_id):
    subject, attempt, original = _proposal(store, run_id)
    subject_digest = digest(subject)
    root = store.root / "candidates"
    ordinary(root)
    root.mkdir(mode=0o700, exist_ok=True)
    verify_inputs_root(root)
    record_path = root / (digest(run_id) + ".json")
    if record_path.exists() or record_path.is_symlink():
        record = read_json(ordinary(record_path))
        if record.get("subject") != subject or record.get("subject_digest") != subject_digest:
            raise ValueError("captured subject changed; stale evidence cannot be reused")
        bundle = verify_input_bundle(store.root / "inputs", record["inputs"])
        if read_json(bundle / "verification.json")["subject"] != subject:
            raise ValueError("captured input subject changed")
        return record
    directory = ordinary(store.root / "attempts" / attempt["id"])
    if str(directory) != attempt["directory"]:
        raise ValueError("accepted attempt directory differs from owned state")
    receipt = read_json(directory / "receipt.json")
    if (digest(receipt) != attempt["receipt_digest"] or receipt.get("attempt_id") != attempt["id"]
            or receipt.get("operation_id") != attempt["operation_id"]
            or receipt.get("spec_digest") != attempt["spec_digest"]
            or receipt.get("status") != "SUCCEEDED" or receipt.get("started") is not True
            or receipt.get("cleanup") is not True or receipt.get("result") != store.status(run_id)["result"]
            or receipt.get("result_digest") != digest(receipt.get("result"))):
        raise ValueError("accepted guardian evidence is missing or changed")
    inputs_root = verify_inputs_root(store.root / "inputs")
    with tempfile.TemporaryDirectory(dir=inputs_root, prefix=".capture-") as temporary:
        stage = Path(temporary)
        bundle = stage / "bundle"
        bundle.mkdir(mode=0o700)
        files = {}
        for name in ("request.json", "task.md", "acceptance.json"):
            files[name] = copy_file(original / name, bundle / name, limit=1 << 20)
        files["candidate.bundle"] = copy_file(directory / "payload/candidate.bundle",
            bundle / "candidate.bundle", subject["bundle_sha256"])
        check_git_subject(bundle / "candidate.bundle", subject, stage / "objects.git", bare=True)
        from durable.intent.policy import check_changes
        policy = acceptance((bundle / "acceptance.json").read_bytes())
        check_changes(stage / "objects.git", subject["base"], subject["commit"], policy)
        request = read_json(bundle / "request.json")
        if policy.get("oracle_files"):
            if "oracle_inputs" not in request:
                raise ValueError("frozen oracle input identity is missing")
            oracle = verify_input_bundle(store.root / "oracles", request["oracle_inputs"])
            for name in policy["oracle_files"]:
                target = bundle / "oracle" / name
                target.parent.mkdir(parents=True, exist_ok=True)
                files["oracle/" + name] = copy_file(oracle / name, target, limit=1 << 20)
        verification_record = {"schema": 1, "subject": subject, "subject_digest": subject_digest,
                               "checks_digest": digest(acceptance((bundle / "acceptance.json").read_bytes()))}
        atomic_json(bundle / "verification.json", verification_record)
        data = (bundle / "verification.json").read_bytes()
        files["verification.json"] = {"sha256": hashlib.sha256(data).hexdigest(), "size": len(data)}
        manifest = {"schema": 1, "files": files}
        input_digest = digest(manifest)
        atomic_json(bundle / "manifest.json", manifest)
        for path in bundle.rglob("*"):
            if path.is_file(): path.chmod(0o400)
        fsync_dir(bundle)
        destination = inputs_root / input_digest
        try:
            bundle.rename(destination)
        except OSError:
            if not destination.exists():
                raise
            verify_input_bundle(inputs_root, input_digest)
        fsync_dir(inputs_root)
        verify_input_bundle(inputs_root, input_digest)
        record = {"schema": 1, "subject": subject, "subject_digest": subject_digest,
                  "inputs": input_digest}
        # Publish without replacing a concurrently recorded capture.
        staged_record = stage / "capture.json"
        atomic_json(staged_record, record)
        try:
            os.link(staged_record, record_path)
        except FileExistsError:
            if read_json(ordinary(record_path)) != record:
                raise ValueError("concurrent capture bound a different subject")
        fsync_dir(root)
    return record


def request_verification(store, execution, run_id):
    if execution is None or execution.backend != "cgroup":
        raise ValueError("verification requires admitted Insula cgroup execution")
    execution.verify()
    if execution.config.get("state") != str(store.root) or execution.config.get("inputs_root") != str(store.root / "inputs"):
        raise ValueError("verification execution must belong to this project")
    record = capture(store, run_id)
    bundle = verify_input_bundle(store.root / "inputs", record["inputs"])
    checks_digest = read_json(bundle / "verification.json")["checks_digest"]
    args = {"subject_digest": record["subject_digest"], "inputs": record["inputs"],
            "checks_digest": checks_digest, "execution_contract": execution.contract}
    verify_id = "verification-" + digest([VERSION, args])
    old = store.db.execute("SELECT 1 FROM runs WHERE id=?", (verify_id,)).fetchone()
    store.submit(verify_id, "verification", VERSION, args, execution.contract)
    return {"run_id": verify_id, **args, "reused": old is not None}


def verification(context, args):
    from durable.runtime.engine import Activity
    return context.activity("verify", Activity(
        argv=("/usr/bin/python3", "-m", "durable.git.verifier"), inputs=args["inputs"],
        cwd="/workspace", input=args, env={"HOME": "/home/bench/run/home"},
        timeout=1800, recovery="manual", max_attempts=1, offline=True))
