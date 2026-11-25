#!/usr/bin/env python3
"""Verify Harp's selected source/contract provenance without executing Lean."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]
SOURCE_COMMIT = "c50a2478e64d25fe8924dba33116f0f7f344bcde"
MODULE = "MathematicalFoundations.Linear"
DECLARATION = MODULE + ".linear_finite_coordinate_reconstruction"
SOURCE_FILES = {"lab/math/lean/lean-toolchain": "lean-toolchain",
                "lab/math/lean/MathematicalFoundations/Linear.lean": "MathematicalFoundations/Linear.lean"}


def read(root: Path, relative: str) -> bytes:
    path = root / relative
    for current in [path, *path.parents]:
        if current == root:
            break
        if current.is_symlink():
            raise ValueError(f"symlinked shared-proof input: {relative}")
    if not path.is_file() or path.stat().st_mode & 0o111:
        raise ValueError(f"missing or executable shared-proof input: {relative}")
    return path.read_bytes()


def check(root: Path = ROOT) -> dict:
    root = root.resolve(strict=True)
    provenance = json.loads(read(root, "lab/math/shared/provenance.json"))
    contract_bytes = read(root, "lab/math/shared/linear.json")
    contract = json.loads(contract_bytes)
    sha = lambda data: hashlib.sha256(data).hexdigest()
    if provenance.get("schema") != 1 or contract.get("schema") != 1:
        raise ValueError("unsupported shared-proof schema")
    if (provenance.get("source_repository") != "https://github.com/phi9t/harp"
            or provenance.get("source_commit") != SOURCE_COMMIT
            or provenance.get("source_root") != "lab/math/lean"
            or provenance.get("contract") != "lab/math/shared/linear.json"):
        raise ValueError("unexpected source origin or contract location")
    if sha(contract_bytes) != provenance.get("contract_sha256"):
        raise ValueError("shared-proof contract digest mismatch")
    records = provenance["source_files"]
    if len(records) != len(SOURCE_FILES) or {item["repository_path"] for item in records} != set(SOURCE_FILES):
        raise ValueError("unexpected shared-proof source selection")
    hashes = {}
    for item in records:
        relative = item["repository_path"]
        if item["projection_path"] != SOURCE_FILES[relative] or item["mode"] != "100644":
            raise ValueError(f"unexpected source mapping: {relative}")
        data = read(root, relative)
        blob = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
        if sha(data) != item["sha256"] or blob != item["git_blob"]:
            raise ValueError(f"shared-proof source digest mismatch: {relative}")
        hashes[item["projection_path"]] = sha(data)
    if read(root, "lab/math/lean/lean-toolchain").decode().strip() != "leanprover/lean4:v4.32.1":
        raise ValueError("shared-proof toolchain changed")
    if contract["toolchain"] != {"path": "lean-toolchain", "sha256": hashes["lean-toolchain"], "value": "leanprover/lean4:v4.32.1"}:
        raise ValueError("contract toolchain does not match its source")
    if contract["sources"] != [{"path": "MathematicalFoundations/Linear.lean", "sha256": hashes["MathematicalFoundations/Linear.lean"]}]:
        raise ValueError("contract sources do not match their provenance")
    if (contract["name"] != "harp_shared_linear"
            or contract["libraries"] != [{"name": "MathematicalFoundations", "roots": [MODULE]}]
            or contract["targets"] != ["MathematicalFoundations"]
            or contract["dependencies"] != ["mathlib"]
            or contract["audit"]["imports"] != [MODULE]):
        raise ValueError("unexpected Harp shared-proof boundary")
    declarations = contract["audit"]["declarations"]
    if (len(declarations) != 1 or set(declarations[0]) != {"name", "type"}
            or declarations[0]["name"] != DECLARATION
            or not isinstance(declarations[0]["type"], str) or not declarations[0]["type"].strip()):
        raise ValueError("unexpected audited theorem selection")
    return {"status": "verified", "source_commit": SOURCE_COMMIT,
            "contract_sha256": sha(contract_bytes), "source_files": len(records),
            "audited_declaration": DECLARATION, "proof_verified": False}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=ROOT)
    args = parser.parse_args()
    try:
        print(json.dumps(check(args.repo_root), sort_keys=True))
        return 0
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(json.dumps({"status": "error", "error": str(error)}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
