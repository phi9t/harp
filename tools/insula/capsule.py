#!/usr/bin/env python3
"""Layered insula capsules: build from a request, run under bubblewrap.

A capsule is an ordered stack of layers plus a run policy. Layers are content
addressed. The root filesystem is a tmpfs assembled by bwrap (bwrap 0.8 has no
overlayfs), so every layer is bound read-only at its own canonical path and
nothing can shadow another layer.

  build  REQUEST.json           build layers, write the capsule record, print it
  run    CAPSULE|REQUEST -- CMD run a command; workspace at /workspace
  show   CAPSULE                print the record
  probe  CAPSULE                check isolation, offline, identity, shadowing
  gc                            remove layers no capsule references

Layer kinds (request "layers" list, in stack order):
  base     {"kind":"base","source":"host-usr"}      host /usr and a minimal /etc (non-hermetic)
  host-ro  {"kind":"host-ro","name":N,"paths":[..],"probe":["cmd",..],"path":[..]}
           host paths bound read-only at the same path; digested by fingerprint (non-hermetic)
  tree     {"kind":"tree","name":N,"source":DIR | "copy":[{"from":F,"to":REL}],"path":[REL..],
            "env":{..},"file_binds":[{"src":REL,"dest":ABS}]}
           copied into the store and bound at /cap/tree/N; digested by content (hermetic)

Store root: $CORENIUS_CAPSULES or ~/.cache/corenius/capsules. Standard library only.
"""

from __future__ import annotations

import argparse
import glob
import hashlib
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import threading
import time
from datetime import datetime, timezone
from pathlib import Path

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


class CapsuleError(Exception):
    pass


def now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def store_root() -> Path:
    root = Path(os.environ.get("CORENIUS_CAPSULES") or Path.home() / ".cache/corenius/capsules")
    (root / "layers").mkdir(parents=True, exist_ok=True)
    (root / "capsules").mkdir(parents=True, exist_ok=True)
    return root


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
    parts = {
        "paths": [path_fingerprint(p) for p in layer["paths"]],
        "binds": [stat_fingerprint(b["src"], b["dest"]) for b in layer.get("binds", [])],
        "probe": [probe_output(c) for c in layer.get("probe", [])],
    }
    return sha(canonical(parts))


# --- layers -------------------------------------------------------------------


def expand(value: str) -> str:
    return os.path.expanduser(os.path.expandvars(value))


def build_tree_layer(layer: dict, root: Path) -> str:
    staging = Path(tempfile.mkdtemp(prefix="stage-", dir=root / "layers"))
    try:
        tree = staging / "tree"
        if "source" in layer:
            source = Path(expand(layer["source"]))
            if not source.is_dir():
                raise CapsuleError(f"layer {layer['name']}: source {source} is not a directory")
            shutil.copytree(source, tree, symlinks=True)
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
        elif kind in ("host-ro", "tree"):
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
    if not layers or layers[0]["kind"] != "base":
        raise CapsuleError("the first layer must be the base")
    return layers


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


def layer_env(layers: list[dict]) -> tuple[list[str], dict]:
    path, env = [], {}
    for layer in layers:
        for entry in layer.get("path", []):
            path.append(entry if entry.startswith("/") else f"{layer['at']}/{entry}")
        for key, value in layer.get("env", {}).items():
            env[key] = value.replace("@AT@", layer.get("at", ""))
    return path, env


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
             ro: list[tuple[str, str]] | None = None) -> list[str]:
    policy = record["policy"]
    net = net or policy["net"]
    gpu = policy["gpu"] if gpu is None else gpu
    argv = ["bwrap", "--die-with-parent", "--new-session", "--unshare-ipc", "--unshare-pid", "--unshare-uts",
            "--unshare-cgroup-try", "--hostname", "corenius-bench"]
    if net == "none":
        argv.append("--unshare-net")
    argv += ["--tmpfs", "/", "--ro-bind", "/usr", "/usr"]
    for link, target in (("/bin", "usr/bin"), ("/lib", "usr/lib"), ("/lib64", "usr/lib64"), ("/sbin", "usr/sbin")):
        argv += ["--symlink", target, link]
    argv += ["--dir", "/etc"]
    etc = list(ETC_FILES) + (["resolv.conf", "hosts", "ssl"] if net == "host" else ["ssl"])
    for name in etc:
        argv += ["--ro-bind-try", f"/etc/{name}", f"/etc/{name}"]
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
    for flag, pairs in (("--bind", rw), ("--ro-bind", ro or [])):
        for host, dest in pairs:
            if not (dest.startswith("/workspace/") or dest.startswith(HOME + "/") or dest.startswith("/run/corenius-tmp/")):
                raise CapsuleError(f"bind destination {dest} must be under /workspace, {HOME} or /run/corenius-tmp")
            argv += [flag, host, dest]
    path, layer_envs = layer_env(record["layers"])
    env = {"HOME": HOME, "TMPDIR": "/run/corenius-tmp", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8",
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
        ro: list[tuple[str, str]] | None = None, stdout=None, stderr=None, on_line=None) -> int:
    verify_host_layers(record)
    workspace = str(Path(workspace).resolve())
    if not Path(workspace).is_dir():
        raise CapsuleError(f"workspace {workspace} is not a directory")
    argv = run_argv(record, workspace, rw, env, ro_workspace, net, gpu, cwd, ro)
    started = time.time()
    reader = None
    if on_line is not None:
        # Stream merged stdout and stderr line by line to the callback (stdout/stderr arguments are ignored).
        stdout, stderr = subprocess.PIPE, subprocess.STDOUT
    proc = subprocess.Popen(argv + ["--"] + command, env={"PATH": "/usr/bin:/bin"}, start_new_session=True,
                            stdout=stdout, stderr=stderr)
    if on_line is not None:
        def pump():
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
