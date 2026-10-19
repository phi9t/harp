"""Payload adapter; writes proposals only, with no access to controller state."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from durable.common import atomic_json, publish_result, read_json
from durable.estate.git import relative_path


def run(argv, cwd):
    return subprocess.run(argv, cwd=cwd, check=True, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE).stdout.decode().strip()


def provider_prompt(inputs, admitted):
    """Materialize only the owner-frozen assignment, never candidate context."""
    task = (inputs / 'task.md').read_bytes()
    checks = (inputs / 'acceptance.json').read_text()
    records = read_json(inputs / 'intent.json')
    assignment = admitted['assignment']
    if (not isinstance(records, dict) or
            any(not isinstance(name, str) or not isinstance(text, str)
                for name, text in records.items()) or
            {name: hashlib.sha256(text.encode('utf-8')).hexdigest()
             for name, text in records.items()} != assignment['records'] or
            records.get(admitted['task_path'], '').encode('utf-8') != task or
            hashlib.sha256(task).hexdigest() != admitted['task_sha256']):
        raise ValueError('frozen intent differs from the admitted assignment')
    if assignment['allowed_paths'] != json.loads(checks)['allowed_paths']:
        raise ValueError('frozen scope differs from the admitted assignment')
    context = {'task_path': admitted['task_path'], 'records': records,
               'allowed_paths': assignment['allowed_paths']}
    return ("Implement this admitted task in the current Git repository. Work only in this "
            "repository and within the admitted allowed_paths. Frozen parent and dependency "
            "records below provide task context; candidate files cannot redefine this assignment. "
            "Do not approve, verify on behalf of the independent verifier, or land changes into "
            "another checkout. The controller will capture your candidate and run independent "
            "checks. Leave source changes ready for capture.\n\nFrozen assignment:\n" +
            json.dumps(context, sort_keys=True, ensure_ascii=False) +
            "\n\nFrozen acceptance inputs:\n" + checks)


def main():
    envelope = json.load(sys.stdin)
    admitted = read_json(Path("/home/bench/input/request.json"))
    from durable.common import digest
    if (digest(admitted) != envelope["input"]["identity"] or
            admitted["provider"] != envelope["input"]["provider"]):
        raise ValueError("activity disagrees with immutable admitted request")
    attempt = Path(envelope["attempt_dir"])
    workspace = attempt / "workspace"
    home = attempt / "home"
    home.mkdir(mode=0o700)
    os.environ.update(HOME=str(home), GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL="/dev/null",
                      GIT_AUTHOR_NAME="Harp worker", GIT_AUTHOR_EMAIL="worker@harp.local",
                      GIT_COMMITTER_NAME="Harp worker", GIT_COMMITTER_EMAIL="worker@harp.local")
    from durable.estate.workspaces import materialize
    materialize('/home/bench/input/source.bundle', admitted['base'], workspace)
    atomic_json(attempt / "progress.json", {"phase": "coding", "attempt_id": envelope["attempt_id"],
                "provider": admitted["provider"], "started_at": time.time()})
    provider_report = None
    if admitted["provider"] == "deterministic":
        fixture = admitted["fixture"] or {"path": "harp-worker.txt", "content": "Harp managed worker\n", "sleep": 0}
        destination = workspace / relative_path(fixture["path"])
        if any(p.is_symlink() for p in (destination, *destination.parents)):
            raise ValueError("fixture edit must not follow a source symlink")
        destination.parent.mkdir(parents=True, exist_ok=True)
        time.sleep(fixture["sleep"])
        destination.write_text(fixture["content"])
    else:
        from durable.jobs.coding.providers import codex, trae
        adapter = {"codex": codex, "trae": trae}[admitted["provider"]]
        prompt = provider_prompt(Path("/home/bench/input"), admitted)
        provider_report = adapter.run(workspace, prompt, admitted, attempt)
    run(["/usr/bin/git", "-c", "core.hooksPath=/dev/null", "add", "-A"], workspace)
    changed = subprocess.run(["/usr/bin/git", "diff", "--cached", "--quiet"], cwd=workspace).returncode
    if changed not in {0, 1}:
        raise ValueError("cannot inspect candidate index")
    if changed:
        run(["/usr/bin/git", "-c", "core.hooksPath=/dev/null", "commit", "-m", "Apply admitted coding task"], workspace)
    run(["/usr/bin/git", "merge-base", "--is-ancestor", admitted["base"], "HEAD"], workspace)
    candidate = run(["/usr/bin/git", "rev-parse", "HEAD"], workspace)
    tree = run(["/usr/bin/git", "rev-parse", "HEAD^{tree}"], workspace)
    output = attempt / "candidate.bundle"
    run(["/usr/bin/git", "bundle", "create", str(output), "HEAD"], workspace)
    with output.open("rb") as stream:
        sha = hashlib.file_digest(stream, "sha256").hexdigest()
    publish_result(envelope, {"schema": 1, "identity": envelope["input"]["identity"],
                   "attempt_id": envelope["attempt_id"], "base": admitted["base"],
                   "provider": admitted["provider"], "commit": candidate, "tree": tree,
                   "bundle": "candidate.bundle", "bundle_sha256": sha,
                   "provider_report": provider_report})


if __name__ == "__main__":
    main()
