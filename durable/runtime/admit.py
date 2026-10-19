"""Prepare and run local durable workflows with admitted Insula activities.

Use as a module from the repository root: python3 -m tools.insula.durable.
Preparing does not submit or execute domain work. Executing delegates to the
admitted runtime CLI and creates no keeper or background service.
"""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from insula.temp_root import temporary_root
from insula import capsule
from durable.common import atomic_json, read_json
from durable.execution import InsulaExecution, snapshot_code


def prepare(args: argparse.Namespace) -> InsulaExecution:
    record = capsule.load_capsule(args.capsule)
    capsule.verify_host_layers(record)
    with tempfile.TemporaryDirectory(dir=temporary_root(), prefix="durable-code-") as staging:
        source = snapshot_code(Path(staging) / "code", args.workflow_source)
        code_digest = capsule.build_tree_layer({"kind": "tree", "name": "durable-code",
                                                "source": str(source)}, capsule.store_root())
    code = capsule.store_root() / "layers" / code_digest / "tree"
    execution = InsulaExecution.create(record, code, args.state, backend=args.backend,
                                      cgroup_root=args.cgroup_root)
    execution.verify()
    if args.execution.exists():
        if read_json(args.execution) != execution.config:
            raise ValueError("execution file already binds another configuration; use a new file")
    else:
        atomic_json(args.execution, execution.config)
    return execution


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    setup = sub.add_parser("prepare")
    setup.add_argument("--capsule", required=True, help="existing capsule digest or request JSON")
    setup.add_argument("--state", type=Path, required=True)
    setup.add_argument("--execution", type=Path, required=True)
    setup.add_argument("--backend", choices=["process-group", "cgroup"], default="process-group")
    setup.add_argument("--cgroup-root", type=Path)
    setup.add_argument("--workflow-source", type=Path, help="additional import-root contents to admit")
    execute = sub.add_parser("exec")
    execute.add_argument("--execution", type=Path, required=True)
    execute.add_argument("arguments", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    try:
        root = temporary_root()
        if args.command == "prepare":
            print(prepare(args).contract)
            return 0
        execution = InsulaExecution(read_json(args.execution))
        execution.verify()
        command = args.arguments[1:] if args.arguments[:1] == ["--"] else args.arguments
        if not command:
            raise ValueError("exec requires runtime CLI arguments after --")
        config = execution.config
        environment = {"PATH": os.defpath, "TMPDIR": str(root), "TMP": str(root), "TEMP": str(root),
                       "PYTHONNOUSERSITE": "1", "PYTHONDONTWRITEBYTECODE": "1",
                       "INSULA_CAPSULES": config["capsule_store"]}
        return subprocess.call([sys.executable, "-m", "durable.runtime", "--state", config["state"],
                                "--contract", execution.contract, "--execution", str(args.execution.resolve()),
                                *command], cwd=config["code"], env=environment)
    except (OSError, ValueError, RuntimeError) as exc:
        print(f"durable Insula: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
