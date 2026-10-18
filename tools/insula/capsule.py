#!/usr/bin/env python3
"""Layered insula capsules: build from a request, run under bubblewrap.

A capsule is an ordered stack of layers plus a run policy. Layers are content
addressed. The root filesystem is a tmpfs assembled by bwrap (bwrap 0.8 has no
overlayfs), so every layer is bound read-only at its own canonical path and
nothing can shadow another layer.

  build  REQUEST.json           build layers, write the capsule record, print it
  run    CAPSULE|REQUEST -- CMD run a command; workspace at /workspace
         (and policy workspace aliases, for upstream layouts like /app and /tests)
  show   CAPSULE                print the record
  probe  CAPSULE                check isolation, offline, identity, shadowing
  gc                            remove layers no capsule references

Layer kinds (request "layers" list, in stack order):
  base     {"kind":"base","source":"host-usr"}      host /usr and a minimal /etc (non-hermetic)
  rootfs   {"kind":"rootfs","name":N,"source":DIR}  prepared root filesystem tree mounted read-only (non-hermetic)
  host-ro  {"kind":"host-ro","name":N,"paths":[..],"probe":["cmd",..],"path":[..]}
           host paths bound read-only at the same path; digested by fingerprint (non-hermetic)
  tree     {"kind":"tree","name":N,"source":DIR | "copy":[{"from":F,"to":REL}],"path":[REL..],
            "env":{..},"file_binds":[{"src":REL,"dest":ABS}],"link_mode":"hardlink"}
           materialized into the store and bound at /cap/tree/N; digested by content (hermetic)

Store root: $CORENIUS_CAPSULES or ~/.cache/corenius/capsules. Standard library only.

Policy may declare workspace_writable_overlays:
  [{"dest":"/etc/app.d","path":".insula/app.d","kind":"dir"},
   {"dest":"/etc/app.conf","path":".insula/app.conf","kind":"file"}]
These bind writable workspace-backed paths over selected absolute paths inside
the insula while keeping the rootfs and dependency bundles immutable.

Policy may also declare user = "root" with cap_add = ["CAP_NET_BIND_SERVICE"]
for rootfs-backed service tasks that must bind a privileged port. Root is mapped
inside an unshared user namespace; the host process still runs as the caller.
Root policies that declare identity.users or identity.groups run bwrap inside a
prebuilt user namespace so benchmark users can be switched to with runuser.
"""

from __future__ import annotations

import argparse
import glob
import hashlib
import json
import os
import pwd
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import threading
import time
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)

DEFAULT_PATH = ["/usr/local/sbin", "/usr/local/bin", "/usr/sbin", "/usr/bin", "/sbin", "/bin"]
FORBIDDEN_ENV = {"LIBRARY_PATH", "LD_LIBRARY_PATH", "LD_PRELOAD", "CPATH", "C_INCLUDE_PATH", "CPLUS_INCLUDE_PATH",
                 "PYTHONPATH", "PYTHONHOME", "GCC_EXEC_PREFIX", "COMPILER_PATH"}
PROXY_ENV = ("HTTP_PROXY", "HTTPS_PROXY", "NO_PROXY", "http_proxy", "https_proxy", "no_proxy")
EXCLUDE_NAMES = {".DS_Store", "__pycache__"}
EXCLUDE_SUFFIXES = {".pyc", ".pyo"}
ETC_FILES = ("passwd", "group", "nsswitch.conf", "ld.so.cache", "alternatives")
HOME = "/home/bench"
POLICY_USERS = {"bench", "root"}
POLICY_CAPS = {"CAP_NET_BIND_SERVICE"}
RESERVED_ALIAS_ROOTS = {
    "/bin", "/cap", "/dev", "/etc", "/home", "/lib", "/lib64", "/proc", "/run", "/sbin", "/usr", "/workspace",
}
IDENTITY_NAME_RE = re.compile(r"[a-z_][a-z0-9_-]{0,31}\$?")


class CapsuleError(Exception):
    pass


def now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def store_root() -> Path:
    root = Path(os.environ.get("CORENIUS_CAPSULES") or Path.home() / ".cache/corenius/capsules")
    (root / "layers").mkdir(parents=True, exist_ok=True)
    (root / "capsules").mkdir(parents=True, exist_ok=True)
    return root


def int_field(line: str, index: int) -> int | None:
    fields = line.split(":")
    if len(fields) <= index:
        return None
    try:
        return int(fields[index])
    except ValueError:
        return None


def validate_identity_name(value: object, field: str) -> str:
    if not isinstance(value, str) or not IDENTITY_NAME_RE.fullmatch(value):
        raise CapsuleError(f"policy.identity {field} must be a Linux user/group name")
    return value


def validate_identity_id(value: object, field: str) -> int:
    if not isinstance(value, int) or value <= 0 or value > 65535:
        raise CapsuleError(f"policy.identity {field} must be an integer from 1 to 65535")
    return value


def validate_identity_path(value: object, field: str) -> str:
    if not isinstance(value, str) or not value.startswith("/") or ":" in value or "\n" in value:
        raise CapsuleError(f"policy.identity {field} must be an absolute path without ':'")
    return value


def validate_identity_text(value: object, field: str) -> str:
    if not isinstance(value, str) or ":" in value or "\n" in value:
        raise CapsuleError(f"policy.identity {field} must be a string without ':'")
    return value


def policy_identity(policy: dict) -> dict[str, list[dict]]:
    raw = policy.get("identity", {})
    if raw in ({}, None):
        return {"users": [], "groups": []}
    if not isinstance(raw, dict):
        raise CapsuleError("policy.identity must be a table")
    raw_users = raw.get("users", [])
    raw_groups = raw.get("groups", [])
    if not isinstance(raw_users, list) or not isinstance(raw_groups, list):
        raise CapsuleError("policy.identity users and groups must be lists")
    users = []
    seen_user_names = set()
    seen_user_ids = set()
    for index, item in enumerate(raw_users):
        if not isinstance(item, dict):
            raise CapsuleError(f"policy.identity.users[{index}] must be a table")
        name = validate_identity_name(item.get("name"), f"users[{index}].name")
        if name in {"root", "bench"}:
            raise CapsuleError(f"policy.identity user {name} is reserved")
        uid = validate_identity_id(item.get("uid"), f"users[{index}].uid")
        gid = validate_identity_id(item.get("gid", uid), f"users[{index}].gid")
        if name in seen_user_names or uid in seen_user_ids:
            raise CapsuleError(f"duplicate policy.identity user {name}/{uid}")
        seen_user_names.add(name)
        seen_user_ids.add(uid)
        users.append({
            "name": name,
            "uid": uid,
            "gid": gid,
            "gecos": validate_identity_text(item.get("gecos", ""), f"users[{index}].gecos"),
            "home": validate_identity_path(item.get("home", f"/home/{name}"), f"users[{index}].home"),
            "shell": validate_identity_path(item.get("shell", "/bin/sh"), f"users[{index}].shell"),
        })
    groups = []
    seen_group_names = set()
    seen_group_ids = set()
    for index, item in enumerate(raw_groups):
        if not isinstance(item, dict):
            raise CapsuleError(f"policy.identity.groups[{index}] must be a table")
        name = validate_identity_name(item.get("name"), f"groups[{index}].name")
        if name in {"root", "bench", "nogroup"}:
            raise CapsuleError(f"policy.identity group {name} is reserved")
        gid = validate_identity_id(item.get("gid"), f"groups[{index}].gid")
        members = item.get("members", [])
        if not isinstance(members, list):
            raise CapsuleError(f"policy.identity.groups[{index}].members must be a list")
        members = [validate_identity_name(member, f"groups[{index}].members") for member in members]
        if name in seen_group_names or gid in seen_group_ids:
            raise CapsuleError(f"duplicate policy.identity group {name}/{gid}")
        seen_group_names.add(name)
        seen_group_ids.add(gid)
        groups.append({"name": name, "gid": gid, "members": members})
    return {"users": users, "groups": groups}


def identity_max_uid_gid(identity: dict[str, list[dict]]) -> tuple[int, int]:
    max_uid = max([user["uid"] for user in identity["users"]] or [0])
    max_gid = max(
        [user["gid"] for user in identity["users"]] + [group["gid"] for group in identity["groups"]] or [0]
    )
    return max_uid, max_gid


def subid_ranges(path: Path, owner: str) -> list[tuple[int, int]]:
    ranges = []
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return []
    owner_ids = {owner, str(os.getuid())}
    for line in lines:
        parts = line.split(":")
        if len(parts) != 3 or parts[0] not in owner_ids:
            continue
        try:
            start = int(parts[1])
            count = int(parts[2])
        except ValueError:
            continue
        if start > 0 and count > 0:
            ranges.append((start, count))
    return ranges


def select_subid_range(kind: str, required_count: int) -> tuple[int, int]:
    try:
        owner = pwd.getpwuid(os.getuid()).pw_name
    except KeyError:
        owner = str(os.getuid())
    ranges = subid_ranges(Path(f"/etc/sub{kind}"), owner)
    for start, count in ranges:
        if count >= required_count:
            return start, count
    raise CapsuleError(
        f"policy.identity requires /etc/sub{kind} to grant at least {required_count} IDs to {owner}"
    )


def identity_userns_prefix(policy: dict, identity: dict[str, list[dict]]) -> list[str]:
    max_uid, max_gid = identity_max_uid_gid(identity)
    if max_uid == 0 and max_gid == 0:
        return []
    if policy_user(policy) != "root":
        raise CapsuleError("policy.identity requires policy.user = root")
    uid_start, _uid_count = select_subid_range("uid", max_uid)
    gid_start, _gid_count = select_subid_range("gid", max_gid)
    return [
        "unshare",
        "--user",
        "--map-root-user",
        f"--map-users={uid_start},1,{max_uid}",
        f"--map-groups={gid_start},1,{max_gid}",
        "--setgroups",
        "allow",
        "--keep-caps",
    ]


def runtime_identity_files(rootfs: Path | None = None, identity: dict | None = None) -> tuple[Path, Path]:
    """Generated passwd/group entries that add the caller while preserving rootfs package users."""
    uid = os.getuid()
    gid = os.getgid()
    identity = identity or {"users": [], "groups": []}
    base_passwd = ""
    base_group = ""
    if rootfs is not None:
        try:
            base_passwd = (rootfs / "etc/passwd").read_text(encoding="utf-8")
        except OSError:
            base_passwd = ""
        try:
            base_group = (rootfs / "etc/group").read_text(encoding="utf-8")
        except OSError:
            base_group = ""
    digest = hashlib.sha256(
        (base_passwd + "\0" + base_group + "\0" + canonical(identity)).encode("utf-8")
    ).hexdigest()[:12]
    root = store_root() / "runtime" / f"uid_{uid}_gid_{gid}_{digest}"
    root.mkdir(parents=True, exist_ok=True)
    passwd = root / "passwd"
    group = root / "group"
    user_names = {user["name"] for user in identity["users"]} | {"bench"}
    user_ids = {user["uid"] for user in identity["users"]}
    group_names = {group["name"] for group in identity["groups"]} | {"bench"}
    group_ids = {group["gid"] for group in identity["groups"]}
    passwd_lines = [
        line for line in base_passwd.splitlines()
        if line and line.split(":", 1)[0] not in user_names and int_field(line, 2) not in user_ids
    ]
    group_lines = [
        line for line in base_group.splitlines()
        if line and line.split(":", 1)[0] not in group_names and int_field(line, 2) not in group_ids
    ]
    if not any(line.startswith("root:") for line in passwd_lines):
        passwd_lines.insert(0, "root:x:0:0:root:/root:/bin/sh")
    if not any(line.startswith("root:") for line in group_lines):
        group_lines.insert(0, "root:x:0:")
    if not any(line.startswith("nogroup:") for line in group_lines):
        group_lines.append("nogroup:x:65534:")
    for entry in identity["groups"]:
        members = ",".join(entry["members"])
        group_lines.append(f"{entry['name']}:x:{entry['gid']}:{members}")
    for entry in identity["users"]:
        passwd_lines.append(
            f"{entry['name']}:x:{entry['uid']}:{entry['gid']}:{entry['gecos']}:{entry['home']}:{entry['shell']}"
        )
    passwd_lines.append(f"bench:x:{uid}:{gid}:Corenius Bench:{HOME}:/bin/sh")
    group_lines.append(f"bench:x:{gid}:")
    passwd.write_text("\n".join(passwd_lines) + "\n", encoding="utf-8")
    group.write_text("\n".join(group_lines) + "\n", encoding="utf-8")
    return passwd, group


def runtime_hosts_file(host_aliases: list[str]) -> Path | None:
    aliases = []
    seen = set()
    for alias in host_aliases:
        if (
            not alias
            or alias in seen
            or any(ch.isspace() for ch in alias)
            or any(ch in alias for ch in "/:")
        ):
            raise CapsuleError(f"host alias {alias!r} must be a non-empty hostname")
        seen.add(alias)
        aliases.append(alias)
    digest = sha("\n".join(sorted(aliases)) or "localhost")[:16]
    root = store_root() / "runtime" / f"hosts_{digest}"
    root.mkdir(parents=True, exist_ok=True)
    hosts = root / "hosts"
    lines = [
        "127.0.0.1 localhost",
        "127.0.1.1 corenius-bench",
    ]
    if aliases:
        lines.append("127.0.0.1 " + " ".join(sorted(aliases)))
    hosts.write_text("\n".join(lines + [""]), encoding="utf-8")
    return hosts


def sha(data: bytes | str) -> str:
    return hashlib.sha256(data.encode() if isinstance(data, str) else data).hexdigest()


def canonical(obj) -> str:
    return json.dumps(obj, sort_keys=True, separators=(",", ":"))


# --- digests ------------------------------------------------------------------


def tree_digest(root: Path) -> str:
    """Deterministic digest of a tree: type, mode, path, and content or link target."""
    h = hashlib.sha256()
    paths = sorted(
        p for p in root.rglob("*")
        if p.name not in EXCLUDE_NAMES and p.suffix not in EXCLUDE_SUFFIXES
    )
    for path in paths:
        rel = path.relative_to(root).as_posix()
        info = path.lstat()
        mode = stat.S_IMODE(info.st_mode)
        if stat.S_ISDIR(info.st_mode):
            h.update(f"D {mode:o} {rel}\n".encode())
        elif stat.S_ISLNK(info.st_mode):
            h.update(f"L {rel} -> {os.readlink(path)}\n".encode())
        elif stat.S_ISREG(info.st_mode):
            fh = hashlib.sha256()
            with path.open("rb") as handle:
                for chunk in iter(lambda: handle.read(1 << 20), b""):
                    fh.update(chunk)
            h.update(f"F {mode:o} {rel} {fh.hexdigest()}\n".encode())
        else:
            h.update(f"O {stat.S_IFMT(info.st_mode):o} {rel}\n".encode())
    return h.hexdigest()


def probe_output(command: str) -> str:
    try:
        result = subprocess.run(command, shell=True, capture_output=True, text=True, timeout=30,
                                env={"PATH": ":".join(DEFAULT_PATH)})
    except (OSError, subprocess.TimeoutExpired) as exc:
        return f"error:{type(exc).__name__}"
    lines = (result.stdout or result.stderr).strip().splitlines()
    return f"rc={result.returncode} {lines[0] if lines else ''}"


def path_fingerprint(path: str) -> dict:
    real = os.path.realpath(path)
    if not os.path.exists(real):
        return {"path": path, "missing": True}
    info = os.stat(real)
    if os.path.isdir(real):
        entries = []
        for name in sorted(os.listdir(real))[:4000]:
            try:
                est = os.lstat(os.path.join(real, name))
            except OSError:
                continue
            entries.append([name, est.st_size, est.st_mtime_ns])
        return {"path": path, "real": real, "dir": sha(canonical(entries))}
    with open(real, "rb") as handle:
        head = handle.read(1 << 20)
    return {"path": path, "real": real, "size": info.st_size, "head": sha(head)}


def stat_fingerprint(src: str, dest: str) -> dict:
    """Existence, type, size and mtime only: bound credential files are never read for digesting."""
    try:
        info = os.stat(src)
    except OSError:
        return {"src": src, "dest": dest, "missing": True}
    return {"src": src, "dest": dest, "dir": stat.S_ISDIR(info.st_mode), "size": info.st_size, "mtime": info.st_mtime_ns}


def host_digest(layer: dict) -> str:
    if layer["kind"] == "base":
        release = Path("/usr/lib/os-release")
        dpkg = Path("/var/lib/dpkg/status")
        parts = {
            "release": sha(release.read_bytes()) if release.exists() else "",
            "dpkg": sha(dpkg.read_bytes()) if dpkg.exists() else "",
            "arch": os.uname().machine,
            "usr": path_fingerprint("/usr/bin"),
        }
        return sha(canonical(parts))
    if layer["kind"] == "rootfs":
        return sha(canonical({"source": layer["source"], "fingerprint": path_fingerprint(layer["source"])}))
    parts = {
        "paths": [path_fingerprint(p) for p in layer["paths"]],
        "binds": [stat_fingerprint(b["src"], b["dest"]) for b in layer.get("binds", [])],
        "probe": [probe_output(c) for c in layer.get("probe", [])],
    }
    return sha(canonical(parts))


# --- layers -------------------------------------------------------------------


def expand(value: str) -> str:
    return os.path.expanduser(os.path.expandvars(value))


def hardlink_or_copy(src: str, dest: str, *, follow_symlinks: bool = True) -> str:
    try:
        os.link(src, dest, follow_symlinks=follow_symlinks)
    except OSError:
        shutil.copy2(src, dest, follow_symlinks=follow_symlinks)
    return dest


def build_tree_layer(layer: dict, root: Path) -> str:
    staging = Path(tempfile.mkdtemp(prefix="stage-", dir=root / "layers"))
    try:
        tree = staging / "tree"
        if "source" in layer:
            source = Path(expand(layer["source"]))
            if not source.is_dir():
                raise CapsuleError(f"layer {layer['name']}: source {source} is not a directory")
            copy_function = hardlink_or_copy if layer.get("link_mode") == "hardlink" else shutil.copy2
            shutil.copytree(source, tree, symlinks=True, copy_function=copy_function)
        else:
            tree.mkdir()
            for item in layer.get("copy", []):
                src = Path(expand(item["from"]))
                dest = tree / item["to"]
                if ".." in Path(item["to"]).parts:
                    raise CapsuleError(f"layer {layer['name']}: copy target escapes the layer")
                if not src.exists():
                    raise CapsuleError(f"layer {layer['name']}: {src} not found")
                dest.parent.mkdir(parents=True, exist_ok=True)
                if src.is_dir():
                    shutil.copytree(src, dest, symlinks=True)
                else:
                    shutil.copy2(src, dest)
        digest = tree_digest(tree)
        final = root / "layers" / digest
        if final.exists():
            return digest
        (staging / "meta.json").write_text(canonical({"name": layer["name"], "kind": "tree", "built": now()}))
        staging.rename(final)
        staging = None
        return digest
    finally:
        if staging is not None:
            shutil.rmtree(staging, ignore_errors=True)


def normalize(request: dict, base_dir: Path | None = None) -> list[dict]:
    layers = []
    names = set()
    for index, raw in enumerate(request.get("layers", [])):
        layer = dict(raw)
        kind = layer.get("kind")
        if kind == "base":
            if layer.get("source", "host-usr") != "host-usr":
                raise CapsuleError("base source must be host-usr")
            layer.setdefault("name", "base")
        elif kind in ("host-ro", "tree", "rootfs"):
            if not layer.get("name"):
                raise CapsuleError(f"layer {index}: {kind} needs a name")
        else:
            raise CapsuleError(f"layer {index}: unknown kind {kind!r}")
        if layer["name"] in names:
            raise CapsuleError(f"duplicate layer name {layer['name']}")
        names.add(layer["name"])
        if kind == "tree":
            layer["at"] = f"/cap/tree/{layer['name']}"
            if "source" in layer:
                source = Path(expand(layer["source"]))
                layer["source"] = str(source if source.is_absolute() or base_dir is None else (base_dir / source).resolve())
        if kind == "rootfs":
            source = Path(expand(layer.get("source", "")))
            source = source if source.is_absolute() or base_dir is None else (base_dir / source).resolve()
            if not source.is_dir():
                raise CapsuleError(f"layer {layer['name']}: rootfs source {source} is not a directory")
            layer["source"] = str(source)
        if kind == "host-ro":
            layer["paths"] = [expand(p) for p in layer.get("paths", [])]
            layer["binds"] = [{"src": expand(b["src"]), "dest": b["dest"]} for b in layer.get("binds", [])]
            for b in layer["binds"]:
                if not (b["dest"].startswith(HOME + "/") and ".." not in Path(b["dest"]).parts):
                    raise CapsuleError(f"layer {layer['name']}: bind dest {b['dest']} must be under {HOME}/")
                if not b["src"].startswith("/"):
                    raise CapsuleError(f"layer {layer['name']}: bind src {b['src']} must be absolute")
            for p in layer["paths"]:
                if not p.startswith("/"):
                    raise CapsuleError(f"layer {layer['name']}: {p} must be absolute")
        layer["path"] = [expand(x) for x in layer.get("path", [])]
        layer["env"] = {k: expand(v) for k, v in layer.get("env", {}).items()}
        layers.append(layer)
    if not layers or layers[0]["kind"] not in ("base", "rootfs"):
        raise CapsuleError("the first layer must be the base or rootfs")
    if any(layer["kind"] == "rootfs" for layer in layers[1:]):
        raise CapsuleError("rootfs must be the first layer")
    return layers


def has_rootfs_layer(layers: list[dict]) -> bool:
    return bool(layers and layers[0]["kind"] == "rootfs")


def host_binds(path: str) -> list[tuple[str, str]]:
    """(source, destination) binds so `path` resolves inside the sandbox.

    A symlink is followed: the real directory is bound where the link points, and the link
    itself is recreated with --symlink unless it lives under the already-bound /usr.
    """
    if os.path.islink(path):
        target = os.readlink(path)
        if not target.startswith("/"):
            target = os.path.normpath(os.path.join(os.path.dirname(path), target))
        target = target.rstrip("/") or "/"
        return [(os.path.realpath(path), target)]
    return [(path, path)]


def mount_points(layers: list[dict]) -> list[tuple[str, str]]:
    points = []
    for layer in layers:
        if layer["kind"] == "tree":
            points.append((layer["name"], layer["at"]))
            points.extend((layer["name"], fb["dest"]) for fb in layer.get("file_binds", []))
        elif layer["kind"] == "host-ro":
            for p in layer["paths"]:
                points.extend((layer["name"], dest) for _, dest in host_binds(p))
    return points


def home_binds(layers: list[dict]) -> list[tuple[str, str]]:
    return [(b["src"], b["dest"]) for layer in layers if layer["kind"] == "host-ro" for b in layer.get("binds", [])]


def check_shadowing(layers: list[dict]) -> None:
    points = mount_points(layers)
    reserved = ["/usr", "/bin", "/lib", "/lib64", "/sbin", "/etc", "/proc", "/dev", "/run/corenius-tmp", "/run", "/home", HOME,
                "/workspace"]
    for name, at in points:
        for other in reserved:
            if at == other or at.startswith(other + "/") and other not in ("/usr", "/home"):
                raise CapsuleError(f"layer {name}: {at} shadows reserved path {other}")
    for i, (name_a, a) in enumerate(points):
        for name_b, b in points[i + 1:]:
            if a == b or a.startswith(b + "/") or b.startswith(a + "/"):
                raise CapsuleError(f"layers {name_a} and {name_b} shadow each other: {a} vs {b}")


def check_workspace_alias_shadowing(layers: list[dict], aliases: list[tuple[str, str]]) -> None:
    for alias, _target in aliases:
        for name, mount in mount_points(layers):
            if alias == mount or mount.startswith(alias + "/") or alias.startswith(mount + "/"):
                raise CapsuleError(f"workspace alias {alias} shadows layer {name}: {mount}")


def layer_env(layers: list[dict]) -> tuple[list[str], dict]:
    path, env = [], {}
    for layer in layers:
        for entry in layer.get("path", []):
            path.append(entry if entry.startswith("/") else f"{layer['at']}/{entry}")
        for key, value in layer.get("env", {}).items():
            env[key] = value.replace("@AT@", layer.get("at", ""))
    return path, env


def validate_workspace_alias(alias: str) -> str:
    path = PurePosixPath(alias)
    if not path.is_absolute() or len(path.parts) != 2 or ".." in path.parts:
        raise CapsuleError(f"workspace alias {alias!r} must be a root-level absolute path like /app")
    value = str(path)
    if value in RESERVED_ALIAS_ROOTS:
        raise CapsuleError(f"workspace alias {value} shadows a reserved capsule path")
    return value


def workspace_aliases(policy: dict) -> list[tuple[str, str]]:
    """Root-level aliases for workspace paths, such as /app -> /workspace."""
    raw_aliases = policy.get("workspace_aliases", [])
    if not isinstance(raw_aliases, list) or any(not isinstance(alias, str) for alias in raw_aliases):
        raise CapsuleError("policy.workspace_aliases must be a list of absolute paths")
    raw_subpath_aliases = policy.get("workspace_path_aliases", {})
    if not isinstance(raw_subpath_aliases, dict) or any(
        not isinstance(alias, str) or not isinstance(rel, str)
        for alias, rel in raw_subpath_aliases.items()
    ):
        raise CapsuleError("policy.workspace_path_aliases must map absolute aliases to relative workspace paths")
    normalized: list[tuple[str, str]] = []
    seen = set()
    for alias in raw_aliases:
        value = validate_workspace_alias(alias)
        if value in seen:
            raise CapsuleError(f"duplicate workspace alias {value}")
        seen.add(value)
        normalized.append((value, "/workspace"))
    for alias, rel in raw_subpath_aliases.items():
        value = validate_workspace_alias(alias)
        rel_path = PurePosixPath(rel)
        if rel_path.is_absolute() or ".." in rel_path.parts or str(rel_path) in {"", "."}:
            raise CapsuleError(f"workspace alias {value} target {rel!r} must be a relative workspace path")
        if value in seen:
            raise CapsuleError(f"duplicate workspace alias {value}")
        seen.add(value)
        normalized.append((value, f"/workspace/{rel_path.as_posix()}"))
    return normalized


def workspace_alias_target_dirs(aliases: list[tuple[str, str]]) -> list[str]:
    dirs = []
    seen = set()
    for _alias, target in aliases:
        path = PurePosixPath(target)
        if path == PurePosixPath("/workspace") or not str(path).startswith("/workspace/"):
            continue
        cur = PurePosixPath("/workspace")
        for part in path.relative_to("/workspace").parts:
            cur = cur / part
            value = str(cur)
            if value not in seen:
                seen.add(value)
                dirs.append(value)
    return dirs


def workspace_writable_overlays(policy: dict) -> list[dict[str, str]]:
    raw = policy.get("workspace_writable_overlays", [])
    if not isinstance(raw, list):
        raise CapsuleError("policy.workspace_writable_overlays must be a list")
    overlays: list[dict[str, str]] = []
    seen = set()
    for index, item in enumerate(raw):
        if not isinstance(item, dict):
            raise CapsuleError(f"policy.workspace_writable_overlays[{index}] must be a table")
        dest = item.get("dest")
        rel = item.get("path")
        kind = item.get("kind", "dir")
        if not isinstance(dest, str) or not isinstance(rel, str):
            raise CapsuleError(f"policy.workspace_writable_overlays[{index}] needs dest and path strings")
        if kind not in {"dir", "file"}:
            raise CapsuleError(f"policy.workspace_writable_overlays[{index}].kind must be dir or file")
        dest_path = PurePosixPath(dest)
        rel_path = PurePosixPath(rel)
        if not dest_path.is_absolute() or ".." in dest_path.parts or str(dest_path) == "/":
            raise CapsuleError(f"writable overlay dest {dest!r} must be an absolute path inside the insula")
        if rel_path.is_absolute() or ".." in rel_path.parts or str(rel_path) in {"", "."}:
            raise CapsuleError(f"writable overlay path {rel!r} must be relative to the workspace")
        if str(dest_path).startswith(("/proc/", "/dev/", "/sys/", "/run/corenius-tmp/", "/workspace/")) or str(dest_path) in {
            "/proc", "/dev", "/sys", "/run/corenius-tmp", "/workspace", "/usr", "/bin", "/lib", "/lib64", "/sbin", "/cap",
        }:
            raise CapsuleError(f"writable overlay dest {dest} targets a protected capsule path")
        if str(dest_path) in seen:
            raise CapsuleError(f"duplicate writable overlay dest {dest_path}")
        for prior in seen:
            prior_path = PurePosixPath(prior)
            if dest_path == prior_path or str(dest_path).startswith(str(prior_path) + "/") or str(prior_path).startswith(str(dest_path) + "/"):
                raise CapsuleError(f"writable overlay destinations shadow each other: {dest_path} vs {prior_path}")
        seen.add(str(dest_path))
        overlays.append({"dest": str(dest_path), "path": rel_path.as_posix(), "kind": kind})
    return overlays


def workspace_overlay_binds(policy: dict, workspace: str) -> list[tuple[str, str]]:
    binds = []
    for overlay in workspace_writable_overlays(policy):
        source = Path(workspace) / overlay["path"]
        if workspace != "@WORKSPACE@":
            source.parent.mkdir(parents=True, exist_ok=True)
            if overlay["kind"] == "dir":
                source.mkdir(parents=True, exist_ok=True)
            else:
                source.touch(exist_ok=True)
        binds.append((str(source), overlay["dest"]))
    return binds


def policy_user(policy: dict) -> str:
    user = policy.get("user", "bench")
    if user not in POLICY_USERS:
        raise CapsuleError("policy.user must be bench or root")
    return user


def policy_cap_add(policy: dict) -> list[str]:
    raw = policy.get("cap_add", [])
    if not isinstance(raw, list) or any(not isinstance(item, str) for item in raw):
        raise CapsuleError("policy.cap_add must be a list of strings")
    caps = []
    seen = set()
    for cap in raw:
        if cap not in POLICY_CAPS:
            raise CapsuleError(f"policy.cap_add contains unsupported capability {cap}")
        if cap not in seen:
            caps.append(cap)
            seen.add(cap)
    if caps and policy_user(policy) != "root":
        raise CapsuleError("policy.cap_add requires policy.user = root")
    return caps


# --- build --------------------------------------------------------------------


def build(request_path: str) -> dict:
    root = store_root()
    request = json.loads(Path(request_path).read_text())
    base_dir = Path(request_path).resolve().parent
    if "extends" in request:
        parent = json.loads((base_dir / request["extends"]).read_text())
        if "extends" in parent:
            raise CapsuleError("extends may be used one level deep")
        request = {**parent, **request, "layers": parent["layers"] + request.get("layers", []),
                   "policy": {**parent.get("policy", {}), **request.get("policy", {})},
                   "env": {**parent.get("env", {}), **request.get("env", {})}}
    layers = normalize(request, base_dir)
    check_shadowing(layers)
    policy = {"net": "none", "gpu": False, "proxy": False, **request.get("policy", {})}
    if policy["net"] not in ("none", "host"):
        raise CapsuleError("policy.net must be none or host")
    aliases = workspace_aliases(policy)
    workspace_writable_overlays(policy)
    policy_user(policy)
    policy_cap_add(policy)
    identity = policy_identity(policy)
    if identity["users"] or identity["groups"]:
        if policy_user(policy) != "root":
            raise CapsuleError("policy.identity requires policy.user = root")
        if not has_rootfs_layer(layers):
            raise CapsuleError("policy.identity requires a rootfs layer")
        policy["identity"] = identity
    else:
        policy.pop("identity", None)
    identity_userns_prefix(policy, identity)
    check_workspace_alias_shadowing(layers, aliases)
    for key in request.get("env", {}):
        if key in FORBIDDEN_ENV:
            raise CapsuleError(f"env {key} is forbidden in a capsule (it breaks bundled toolchains)")
    recorded = []
    for layer in layers:
        entry = {k: v for k, v in layer.items()}
        if layer["kind"] == "tree":
            entry["digest"] = build_tree_layer(layer, root)
            entry["hermetic"] = True
        else:
            entry["digest"] = host_digest(layer)
            entry["hermetic"] = False
        recorded.append(entry)
    plan = run_argv({"layers": recorded, "policy": policy, "env": request.get("env", {})}, "@WORKSPACE@", [], {}, False, gpu=False)
    plan_digest = sha(canonical(plan))
    stack = [f"{l['kind']}:{l['name']}:{l['digest']}" for l in recorded]
    capsule_digest = sha(canonical({"stack": stack, "plan": plan_digest, "policy": policy, "env": request.get("env", {})}))
    record = {
        "schema": 1,
        "name": request.get("name", "capsule"),
        "capsule_digest": capsule_digest,
        "plan_digest": plan_digest,
        "layers": recorded,
        "policy": policy,
        "env": request.get("env", {}),
        "hermetic": all(l["hermetic"] for l in recorded),
        "non_hermetic_layers": [l["name"] for l in recorded if not l["hermetic"]],
        "host": {"kernel": os.uname().release, "arch": os.uname().machine},
        "built": now(),
    }
    (root / "capsules" / f"{capsule_digest}.json").write_text(json.dumps(record, indent=2) + "\n")
    return record


def load_capsule(ref: str) -> dict:
    root = store_root()
    if ref.endswith(".json") and Path(ref).is_file():
        return build(ref)
    matches = sorted((root / "capsules").glob(f"{ref}*.json"))
    if len(matches) != 1:
        raise CapsuleError(f"capsule {ref!r}: {len(matches)} matches")
    return json.loads(matches[0].read_text())


# --- run ----------------------------------------------------------------------


def gpu_devices() -> list[str]:
    return sorted(glob.glob("/dev/nvidia*")) + sorted(glob.glob("/dev/nvidia-caps/*"))


def run_argv(record: dict, workspace: str, rw: list[tuple[str, str]], extra_env: dict, ro_workspace: bool,
             net: str | None = None, gpu: bool | None = None, cwd: str = "/workspace",
             ro: list[tuple[str, str]] | None = None, host_aliases: list[str] | None = None) -> list[str]:
    policy = record["policy"]
    net = net or policy["net"]
    gpu = policy["gpu"] if gpu is None else gpu
    aliases = workspace_aliases(policy)
    user = policy_user(policy)
    cap_add = policy_cap_add(policy)
    identity = policy_identity(policy)
    if (identity["users"] or identity["groups"]) and not has_rootfs_layer(record["layers"]):
        raise CapsuleError("policy.identity requires a rootfs layer")
    argv = [*identity_userns_prefix(policy, identity), "bwrap", "--die-with-parent", "--new-session", "--unshare-ipc", "--unshare-pid", "--unshare-uts",
            "--unshare-cgroup-try", "--hostname", "corenius-bench"]
    if user == "root" and not (identity["users"] or identity["groups"]):
        argv += ["--unshare-user", "--uid", "0", "--gid", "0"]
    for cap in cap_add:
        argv += ["--cap-add", cap]
    if net == "none":
        argv.append("--unshare-net")
    argv += ["--tmpfs", "/"]
    rootfs = record["layers"][0] if record["layers"] and record["layers"][0]["kind"] == "rootfs" else None
    alias_roots = {PurePosixPath(alias).parts[1] for alias, _target in aliases}
    if rootfs:
        root = Path(rootfs["source"])
        private_roots = {"cap", "dev", "home", "proc", "run", "workspace"}
        for child in sorted(root.iterdir(), key=lambda path: path.name):
            if child.name in private_roots or child.name in alias_roots or child.name == "etc":
                continue
            dest = f"/{child.name}"
            if child.is_symlink():
                argv += ["--symlink", os.readlink(child), dest]
            else:
                argv += ["--ro-bind", str(child), dest]
        argv += ["--dir", "/etc"]
        root_etc = root / "etc"
        if root_etc.exists():
            for child in sorted(root_etc.iterdir(), key=lambda path: path.name):
                if child.name in {"group", "hosts", "passwd"}:
                    continue
                dest = f"/etc/{child.name}"
                if child.is_symlink():
                    argv += ["--symlink", os.readlink(child), dest]
                elif child.is_dir() or child.is_file():
                    argv += ["--ro-bind", str(child), dest]
        passwd, group = runtime_identity_files(root, identity)
        argv += ["--ro-bind", str(passwd), "/etc/passwd", "--ro-bind", str(group), "/etc/group"]
    else:
        argv += ["--ro-bind", "/usr", "/usr"]
        for link, target in (("/bin", "usr/bin"), ("/lib", "usr/lib"), ("/lib64", "usr/lib64"), ("/sbin", "usr/sbin")):
            argv += ["--symlink", target, link]
        argv += ["--dir", "/etc"]
        etc = list(ETC_FILES) + (["resolv.conf", "ssl"] if net == "host" else ["ssl"])
        for name in etc:
            argv += ["--ro-bind-try", f"/etc/{name}", f"/etc/{name}"]
    hosts = runtime_hosts_file(host_aliases or [])
    if hosts is not None:
        argv += ["--ro-bind", str(hosts), "/etc/hosts"]
    argv += ["--proc", "/proc", "--dev", "/dev", "--tmpfs", "/run", "--tmpfs", "/run/corenius-tmp", "--tmpfs", "/home",
             "--dir", HOME, "--dir", "/cap"]
    if gpu:
        devices = gpu_devices()
        if not devices:
            raise CapsuleError("gpu requested but no /dev/nvidia* device exists")
        for dev in devices:
            argv += ["--dev-bind", dev, dev]
        # NVML and the CUDA runtime read PCI and driver state from sysfs; without it they crash silently.
        argv += ["--ro-bind", "/sys", "/sys"]
    for layer in record["layers"]:
        if layer["kind"] == "rootfs":
            continue
        if layer["kind"] == "tree":
            argv += ["--ro-bind", str(store_root() / "layers" / layer["digest"] / "tree"), layer["at"]]
            for fb in layer.get("file_binds", []):
                argv += ["--ro-bind", str(store_root() / "layers" / layer["digest"] / "tree" / fb["src"]), fb["dest"]]
        elif layer["kind"] == "host-ro":
            for p in layer["paths"]:
                for src, dest in host_binds(p):
                    argv += ["--ro-bind", src, dest]
                    if dest != p and not p.startswith("/usr/"):
                        argv += ["--symlink", dest, p]
    for src, dest in home_binds(record["layers"]):
        argv += ["--ro-bind", src, dest]
    argv += ["--ro-bind" if ro_workspace else "--bind", workspace, "/workspace"]
    if not ro_workspace:
        for target_dir in workspace_alias_target_dirs(aliases):
            argv += ["--dir", target_dir]
    for alias, target in aliases:
        argv += ["--symlink", target, alias]
    for host, dest in workspace_overlay_binds(policy, workspace):
        argv += ["--bind", host, dest]
    for flag, pairs in (("--bind", rw), ("--ro-bind", ro or [])):
        for host, dest in pairs:
            if not (dest.startswith("/workspace/") or dest.startswith(HOME + "/") or dest.startswith("/run/corenius-tmp/")):
                raise CapsuleError(f"bind destination {dest} must be under /workspace, {HOME} or /run/corenius-tmp")
            argv += [flag, host, dest]
    path, layer_envs = layer_env(record["layers"])
    env = {"HOME": HOME, "TMPDIR": "/run/corenius-tmp", "TMUX_TMPDIR": "/run/corenius-tmp",
           "TMPPREFIX": "/run/corenius-tmp/zsh", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8",
           "PATH": ":".join(path + DEFAULT_PATH), "CORENIUS_IN_INSULA": "1",
           "CORENIUS_CAPSULE": record.get("capsule_digest", "@DIGEST@")[:16], "PWD": cwd}
    env.update(layer_envs)
    env.update(record.get("env", {}))
    env.update(extra_env)
    if env.get("TMPDIR") != "/run/corenius-tmp":
        raise CapsuleError("capsule TMPDIR is fixed to its private temporary mount")
    if net == "host" and policy.get("proxy"):
        env.update({k: os.environ[k] for k in PROXY_ENV if k in os.environ})
    for key in env:
        if key in FORBIDDEN_ENV:
            raise CapsuleError(f"env {key} is forbidden in a capsule")
    argv.append("--clearenv")
    for key in sorted(env):
        argv += ["--setenv", key, env[key]]
    argv += ["--chdir", cwd]
    return argv


def verify_host_layers(record: dict) -> None:
    for layer in record["layers"]:
        if not layer["hermetic"] and host_digest(layer) != layer["digest"]:
            raise CapsuleError(f"layer {layer['name']} is stale: the host changed since the capsule was built; rebuild it")
        if layer["kind"] == "tree" and not (store_root() / "layers" / layer["digest"] / "tree").is_dir():
            raise CapsuleError(f"layer {layer['name']} is missing from the store; rebuild the capsule")


def run(record: dict, command: list[str], workspace: str, rw: list[tuple[str, str]], env: dict, net: str | None,
        gpu: bool | None, ro_workspace: bool, timeout: float | None, cwd: str, record_path: str | None,
        ro: list[tuple[str, str]] | None = None, stdout=None, stderr=None, on_line=None,
        host_aliases: list[str] | None = None) -> int:
    verify_host_layers(record)
    workspace = str(Path(workspace).resolve())
    if not Path(workspace).is_dir():
        raise CapsuleError(f"workspace {workspace} is not a directory")
    argv = run_argv(record, workspace, rw, env, ro_workspace, net, gpu, cwd, ro, host_aliases=host_aliases)
    started = time.time()
    reader = None
    stream_output = on_line is not None
    if stream_output:
        # Stream merged stdout and stderr line by line to the callback (stdout/stderr arguments are ignored).
        stdout, stderr = subprocess.PIPE, subprocess.STDOUT
    proc = subprocess.Popen(argv + ["--"] + command, env={"PATH": "/usr/bin:/bin"}, start_new_session=True,
                            stdout=stdout, stderr=stderr)
    if stream_output:
        def pump():
            assert proc.stdout is not None
            for raw in iter(proc.stdout.readline, b""):
                on_line(raw.decode("utf-8", "replace").rstrip("\n"))

        reader = threading.Thread(target=pump, daemon=True)
        reader.start()
    timed_out = False
    try:
        code = proc.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out = True
        os.killpg(os.getpgid(proc.pid), 9)
        code = proc.wait()
    if reader is not None:
        reader.join(timeout=10)
    if stream_output and proc.stdout is not None:
        proc.stdout.close()
    if record_path:
        Path(record_path).write_text(json.dumps({
            "capsule_digest": record["capsule_digest"], "plan_digest": record["plan_digest"],
            "hermetic": record["hermetic"], "non_hermetic_layers": record["non_hermetic_layers"],
            "net": net or record["policy"]["net"], "gpu": bool(record["policy"]["gpu"] if gpu is None else gpu),
            "command": command, "exit_code": code, "timed_out": timed_out,
            "elapsed_seconds": round(time.time() - started, 3), "finished": now(),
        }, indent=2) + "\n")
    return 124 if timed_out else code


# --- probe --------------------------------------------------------------------

PROBE_SCRIPT = r"""
set -u
out() { printf '%s %s\n' "$1" "$2"; }
if ( : > /usr/.probe ) 2>/dev/null; then out usr_readonly FAIL; else out usr_readonly PASS; fi
if ( : > /workspace/.probe && rm -f /workspace/.probe ) 2>/dev/null; then out workspace_writable PASS; else out workspace_writable FAIL; fi
leak=0
for entry in $(ls -A "$PROBE_HOST_HOME" 2>/dev/null); do
  case ",$PROBE_HOME_ALLOWED," in *",$entry,"*) ;; *) leak=1 ;; esac
done
if [ "$leak" -eq 0 ]; then out host_home_hidden PASS; else out host_home_hidden FAIL; fi
n=$(ls /proc | grep -c '^[0-9][0-9]*$'); if [ "$n" -le 8 ]; then out pid_isolated PASS; else out pid_isolated FAIL; fi
ifaces=$(tail -n +3 /proc/net/dev | cut -d: -f1 | tr -d ' ' | grep -v '^lo$' | wc -l)
if [ "$PROBE_NET" = none ]; then if [ "$ifaces" -eq 0 ]; then out offline PASS; else out offline FAIL; fi; else out offline SKIP; fi
bad=$(env | cut -d= -f1 | grep -E '^(LIBRARY_PATH|LD_LIBRARY_PATH|LD_PRELOAD|PYTHONPATH|SPACK_.*|CPATH)$' | wc -l)
if [ "$bad" -eq 0 ]; then out env_clean PASS; else out env_clean FAIL; fi
if [ "${CORENIUS_IN_INSULA:-}" = 1 ]; then out marker PASS; else out marker FAIL; fi
"""


def probe(record: dict) -> dict:
    results = {}
    selected = os.environ.get("TMPDIR")
    if not selected or not Path(selected).is_absolute():
        raise CapsuleError("run with tools/temp_root.py or set an owned absolute TMPDIR")
    parent = Path(selected).resolve()
    forbidden = Path("/") / "tmp"
    if parent == forbidden or forbidden in parent.parents:
        raise CapsuleError("TMPDIR is forbidden")
    with tempfile.TemporaryDirectory(dir=parent) as ws:
        home = str(Path.home())
        # A layer mounted under the host home makes its own top-level directory appear there; anything else is a leak.
        allowed = {Path(at).relative_to(home).parts[0] for _, at in mount_points(record["layers"])
                   if at == home or at.startswith(home + "/")} - {"."}
        env = {"PROBE_HOST_HOME": home, "PROBE_HOME_ALLOWED": ",".join(sorted(allowed)),
               "PROBE_NET": record["policy"]["net"]}
        argv = run_argv(record, ws, [], env, False)
        proc = subprocess.run(argv + ["--", "/bin/sh", "-c", PROBE_SCRIPT], capture_output=True, text=True,
                              env={"PATH": "/usr/bin:/bin"}, timeout=60)
    for line in proc.stdout.splitlines():
        name, _, verdict = line.partition(" ")
        results[name] = verdict
    if proc.returncode != 0 and not results:
        results["run"] = f"FAIL {proc.stderr.strip()[:200]}"
    try:
        verify_host_layers(record)
        results["identity"] = "PASS"
    except CapsuleError as exc:
        results["identity"] = f"FAIL {exc}"
    try:
        check_shadowing(record["layers"])
        results["shadowing"] = "PASS"
    except CapsuleError as exc:
        results["shadowing"] = f"FAIL {exc}"
    for layer in record["layers"]:
        if layer["kind"] == "tree":
            actual = tree_digest(store_root() / "layers" / layer["digest"] / "tree")
            if actual != layer["digest"]:
                results["identity"] = f"FAIL layer {layer['name']} content changed"
    results["hermetic"] = "PASS" if record["hermetic"] else "INFO non-hermetic: " + ",".join(record["non_hermetic_layers"])
    results["ok"] = all(v.split()[0] in ("PASS", "SKIP", "INFO") for v in results.values())
    return results


def gc() -> list[str]:
    root = store_root()
    used = set()
    for path in (root / "capsules").glob("*.json"):
        for layer in json.loads(path.read_text())["layers"]:
            if layer["kind"] == "tree":
                used.add(layer["digest"])
    removed = []
    for path in (root / "layers").iterdir():
        if path.name not in used:
            shutil.rmtree(path, ignore_errors=True)
            removed.append(path.name)
    return removed


# --- cli ----------------------------------------------------------------------


def parse_rw(items: list[str]) -> list[tuple[str, str]]:
    out = []
    for item in items:
        host, sep, dest = item.partition(":")
        if not sep:
            raise CapsuleError(f"--rw {item!r} must be HOST:DEST")
        out.append((str(Path(host).resolve()), dest))
    return out


def main(argv: list[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    command: list[str] = []
    if "--" in argv:
        split = argv.index("--")
        argv, command = argv[:split], argv[split + 1:]
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("build").add_argument("request")
    r = sub.add_parser("run")
    r.add_argument("capsule")
    r.add_argument("--workspace", default=".")
    r.add_argument("--ro-workspace", action="store_true")
    r.add_argument("--rw", action="append", default=[])
    r.add_argument("--ro", action="append", default=[])
    r.add_argument("--env", action="append", default=[])
    r.add_argument("--net", choices=["none", "host"])
    r.add_argument("--gpu", action="store_true", default=None)
    r.add_argument("--timeout", type=float)
    r.add_argument("--cwd", default="/workspace")
    r.add_argument("--record")
    sub.add_parser("show").add_argument("capsule")
    sub.add_parser("probe").add_argument("capsule")
    sub.add_parser("gc")
    args = p.parse_args(argv)
    try:
        if args.cmd == "build":
            print(json.dumps(build(args.request), indent=2))
        elif args.cmd == "show":
            print(json.dumps(load_capsule(args.capsule), indent=2))
        elif args.cmd == "gc":
            print(json.dumps({"removed": gc()}))
        elif args.cmd == "probe":
            result = probe(load_capsule(args.capsule))
            print(json.dumps(result, indent=2))
            return 0 if result["ok"] else 1
        elif args.cmd == "run":
            if not command:
                raise CapsuleError("run needs a command after --")
            env = dict(item.partition("=")[::2] for item in args.env)
            return run(load_capsule(args.capsule), command, args.workspace, parse_rw(args.rw), env, args.net,
                       args.gpu, args.ro_workspace, args.timeout, args.cwd, args.record, parse_rw(args.ro))
    except CapsuleError as exc:
        print(f"capsule: {exc}", file=sys.stderr)
        return 64
    return 0


if __name__ == "__main__":
    sys.exit(main())
