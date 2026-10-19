"""Owner-admitted change scope and independently frozen oracle inputs."""
import json
from fnmatch import fnmatchcase
from pathlib import PurePosixPath


def validate_policy(value):
    paths = value.get("allowed_paths")
    if not isinstance(paths, list) or any(not isinstance(p, str) or not p or
            p.startswith("/") or "\\" in p or "\x00" in p or
            any(x in {"..", ".git"} for x in PurePosixPath(p).parts) for p in paths):
        raise ValueError("acceptance requires explicit allowed_paths change policy")
    oracles = value.get("oracle_files", [])
    if (not isinstance(oracles, list) or any(not isinstance(p, str) for p in oracles)
            or len(set(oracles)) != len(oracles)):
        raise ValueError("oracle_files must be unique committed relative paths")
    from durable.estate.git import relative_path
    for path in oracles:
        relative_path(path)
    for argv in value["checks"]:
        # Qualify a small concrete oracle interface. Arbitrary executables,
        # env/shell wrappers and candidate-relative interpreters are not oracles.
        python = argv[0] == "/usr/bin/python3"
        if argv[0] not in {"/usr/bin/python3", "/usr/bin/true", "/usr/bin/false", "/usr/bin/test", "/bin/test"}:
            raise ValueError("unqualified oracle executable; use the pinned system Python oracle")
        if python and (len(argv) < 3 or argv[1] != "-I"):
            raise ValueError("Python oracles require -I to exclude candidate import paths")
        if python:
            entry = argv[2]
            inline = entry == "-c" and len(argv) >= 4
            frozen = entry.startswith("/oracle/") and entry[len("/oracle/"):] in oracles
            if not inline and not frozen:
                raise ValueError("Python oracle must be pinned inline code or a frozen /oracle/ script")
        for arg in argv[1:]:
            if arg.startswith("/oracle/") and arg[len("/oracle/"):] not in oracles:
                raise ValueError("oracle argument does not name a frozen oracle_file")
    return value


def check_changes(repository, base, candidate, policy):
    from durable.estate.git import git
    paths = git(repository, "diff", "--name-only", "--no-renames", "-z", base, candidate).split(b"\0")
    changed = [p.decode("utf-8") for p in paths if p]
    rejected = [p for p in changed if not any(fnmatchcase(p, pattern) for pattern in policy["allowed_paths"])]
    if rejected:
        raise ValueError("candidate exceeds admitted change policy: " + ", ".join(rejected))
    return changed


def acceptance(data):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("duplicate acceptance field")
            result[key] = value
        return result
    value = json.loads(data, object_pairs_hook=unique)
    if not isinstance(value, dict) or not {"checks", "allowed_paths"} <= set(value) or set(value) - {"checks", "allowed_paths", "oracle_files"}:
        raise ValueError("acceptance requires checks and an explicit allowed_paths policy")
    if not isinstance(value["checks"], list) or not 1 <= len(value["checks"]) <= 32:
        raise ValueError("acceptance requires one through 32 checks")
    for argv in value["checks"]:
        if (not isinstance(argv, list) or not argv or
                any(not isinstance(a, str) or not a or "\x00" in a for a in argv)):
            raise ValueError("acceptance checks must be nonempty argv arrays")
    return validate_policy(value)
