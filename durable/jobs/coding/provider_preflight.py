"""Bounded, offline metadata probes of sealed trusted provider executables.

This is setup preflight, not activity scheduling. Process-group cancellation and
bubblewrap parent/PID-namespace lifetime are used for trusted metadata commands;
they do not replace guardian/cgroup ownership for arbitrary coding workloads.
"""
from __future__ import annotations

import copy
import hashlib
import os
from pathlib import Path
import re
import selectors
import signal
import stat
import subprocess
import time

from insula import capsule

TIMEOUT_SECONDS = 30
MAX_OUTPUT = 64 * 1024
COMMON_FLAGS = {"--ignore-user-config", "--ignore-rules", "--ephemeral",
                "--dangerously-bypass-approvals-and-sandbox", "--json",
                "--output-last-message", "--cd", "--model"}
PREFIX = {
    "codex": ["--no-daemon", "--enable", "code_mode_host", "--disable", "apps", "--disable", "plugins"],
    "trae": ["--disable", "daemon_auto_start", "--disable", "hooks", "--disable", "plugins",
             "-c", 'model_provider="trae"'],
}


def _directory(path, private=False):
    path = Path(path)
    if not path.is_absolute() or any(p.is_symlink() for p in (path, *path.parents)):
        raise ValueError("preflight directories must be absolute and have no symlink ancestry")
    info = path.stat()
    if (not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid()
            or (private and info.st_mode & 0o077) or any(path.iterdir())):
        raise ValueError("preflight requires empty owned directories and private scratch")
    return path


def _metadata(argv, deadline):
    process = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, env={"PATH": "/usr/bin:/bin"},
                               start_new_session=True)
    data = bytearray()
    try:
        with selectors.DefaultSelector() as selected:
            selected.register(process.stdout, selectors.EVENT_READ)
            selected.register(process.stderr, selectors.EVENT_READ)
            while selected.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise ValueError("provider metadata probe exceeded its time limit")
                for key, _ in selected.select(remaining):
                    block = os.read(key.fileobj.fileno(), MAX_OUTPUT + 1 - len(data))
                    if not block:
                        selected.unregister(key.fileobj)
                    else:
                        data.extend(block)
                        if len(data) > MAX_OUTPUT:
                            raise ValueError("provider metadata output exceeds 64 KiB")
        observed = None
        while observed is None:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise ValueError("provider metadata probe exceeded its time limit")
            # Observe without reaping: the child PID stays reserved until its
            # owned process group is killed in finally, avoiding PID reuse.
            observed = os.waitid(os.P_PID, process.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
            if observed is None:
                time.sleep(min(0.005, remaining))
        if observed.si_code != os.CLD_EXITED or observed.si_status:
            raise ValueError("provider metadata command failed")
        return bytes(data).decode("utf-8")
    finally:
        # A metadata command must not leave its trusted process group alive,
        # including a descendant retaining an output pipe after parent exit.
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait(timeout=2)
        process.stdout.close()
        process.stderr.close()


def _sealed(record):
    layers = record.get("layers", [])
    native = [layer for layer in layers if layer.get("kind") == "rootfs"]
    providers = [layer for layer in layers if layer.get("kind") == "tree" and layer.get("name") == "providers"]
    if len(native) != 1 or len(providers) != 1:
        raise ValueError("preflight requires one admitted native rootfs and sealed provider tree")
    layer = providers[0]
    names = {item["src"] for item in layer.get("file_binds", [])}
    if not {"codex", "codex-code-mode-host"} <= names or names - {"codex", "codex-code-mode-host", "trae"}:
        raise ValueError("Codex and its code-mode companion are required")
    expected = {name: "/cap/providers/" + name for name in names}
    if {item["src"]: item["dest"] for item in layer.get("file_binds", [])} != expected:
        raise ValueError("provider and code-mode companion mappings are incomplete")
    tree = capsule.store_root() / "layers" / layer["digest"] / "tree"
    if tree.is_symlink() or {p.name for p in tree.iterdir()} != names or capsule.tree_digest(tree) != layer["digest"]:
        raise ValueError("sealed provider tree changed")
    hashes = {}
    for name in sorted(names):
        path = tree / name
        info = path.lstat()
        if not stat.S_ISREG(info.st_mode) or not info.st_mode & 0o111:
            raise ValueError("provider and companion must be regular executable files")
        with path.open("rb") as source:
            hashes[name] = hashlib.file_digest(source, "sha256").hexdigest()
    return native[0], layer, hashes


def _required(provider):
    return COMMON_FLAGS | {"--disable"} | ({"--no-daemon", "--enable", "--color"} if provider == "codex" else {"-c"})


def verify_providers(record: dict, descriptors: dict) -> None:
    """Verify retained version/capability evidence against sealed bytes, with no execution."""
    _, _, hashes = _sealed(record)
    if not isinstance(descriptors, dict) or set(descriptors) != set(hashes) - {"codex-code-mode-host"}:
        raise ValueError("metadata descriptors must match the admitted provider set")
    for provider, item in descriptors.items():
        fields = {"provider", "version", "capabilities", "binary_sha256"}
        if provider == "codex":
            fields.add("companions")
        if (not isinstance(item, dict) or set(item) != fields or item["provider"] != provider
                or not isinstance(item["version"], str) or len(item["version"]) > 80
                or not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?", item["version"])
                or item["capabilities"] != sorted(_required(provider))
                or item["binary_sha256"] != hashes[provider]):
            raise ValueError("provider metadata or sealed executable changed")
        if provider == "codex" and item["companions"] != [{
                "path": "/cap/providers/codex-code-mode-host", "sha256": hashes["codex-code-mode-host"],
                "capabilities": ["--listen"]}]:
            raise ValueError("code-mode companion metadata or sealed executable changed")


def probe_providers(record: dict, workspace: Path, scratch: Path) -> dict:
    workspace = _directory(workspace)
    scratch = _directory(scratch, private=True)
    if workspace.is_relative_to(scratch) or scratch.is_relative_to(workspace):
        raise ValueError("preflight workspace and scratch must be separate directories")
    native, layer, hashes = _sealed(record)
    narrowed = copy.deepcopy(record)
    narrowed["layers"] = [copy.deepcopy(native), copy.deepcopy(layer)]
    for item in narrowed["layers"]:
        item.pop("env", None)
        item.pop("path", None)
    narrowed.pop("env", None)
    narrowed["policy"] = {"net": "none", "gpu": False, "proxy": False}
    capsule.verify_host_layers(narrowed)
    deadline = time.monotonic() + TIMEOUT_SECONDS
    results = {}

    def command(provider, label, argv):
        directory = scratch / (provider + "-" + label)
        directory.mkdir(mode=0o700)
        (directory / "home").mkdir(mode=0o700)
        inside = "/home/bench/run/" + directory.name
        environment = {"HOME": inside + "/home", "CODEX_HOME": inside + "/home/.codex",
                       "TRAE_HOME": inside + "/home/.trae", "TRAECLI_HOME": inside + "/home/.trae/cli",
                       "PYTHONNOUSERSITE": "1", "PYTHONDONTWRITEBYTECODE": "1", "NO_COLOR": "1"}
        launch = capsule.run_argv(narrowed, str(workspace), [(str(scratch), "/home/bench/run")],
                                  environment, True, net="none", cwd="/workspace")
        return _metadata([*launch, "--", *argv], deadline)

    for provider in sorted(set(hashes) - {"codex-code-mode-host"}):
        executable = "/cap/providers/" + provider
        prefix = [executable, *PREFIX[provider]]
        version_text = command(provider, "version", [*prefix, "--version"])
        label = "codex-cli" if provider == "codex" else "traecli"
        versions = re.findall(r"(?m)^" + label + r" ([0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?)\s*$", version_text)
        if len(versions) != 1 or len(versions[0]) > 80:
            raise ValueError("provider metadata did not report one supported version string")
        help_text = command(provider, "help", [*prefix, "exec", "--help"])
        help_text += command(provider, "global-help", [*prefix, "--help"])
        required = _required(provider)
        if any(not re.search(r"(?<![\w-])" + re.escape(flag) + r"(?=[\s=,<]|$)", help_text) for flag in required):
            raise ValueError("provider help lacks a required adapter capability")
        results[provider] = {"provider": provider, "version": versions[0],
                             "capabilities": sorted(required), "binary_sha256": hashes[provider]}
    companion = command("codex", "companion-help", ["/cap/providers/codex-code-mode-host", "--help"])
    if "--listen" not in companion:
        raise ValueError("code-mode companion lacks its required transport capability")
    results["codex"]["companions"] = [{"path": "/cap/providers/codex-code-mode-host",
                                       "sha256": hashes["codex-code-mode-host"], "capabilities": ["--listen"]}]
    verify_providers(record, results)
    return results
