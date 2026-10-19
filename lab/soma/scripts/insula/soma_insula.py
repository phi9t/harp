#!/usr/bin/env python3
"""Selected-rootfs Insula authority helpers for Soma benchmark runners.

This module is intentionally usable both as an importable test target and as a
direct script from the thin shell wrappers in this repository.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Iterable, Sequence


PRODUCTION_REQUIRED_TOOLS = [
    "emacs",
    "emacsclient",
    "python3",
    "jupyter",
    "pylsp",
    "pyright",
    "pyright-langserver",
    "clangd",
    "clang",
    "clang++",
    "gcc",
    "g++",
    "ldd",
    "rust-analyzer",
    "rustc",
    "cargo",
    "git",
    "tmux",
    "sh",
    "bash",
]

NETWORK_NOTE = (
    "Network policy is task-selectable. Hermeticity means rootfs-owned "
    "tools/packages/deps plus ABI accounting, not mandatory network "
    "disablement for every phase."
)
RUN_ID_RE = re.compile(r"^[A-Za-z0-9_.:-]+$")
EXTRA_ENV_ALLOWLIST = frozenset(
    {
        "LSP_USE_PLISTS",
    }
)
EXTRA_PLAN_STRING_ALLOWLIST = frozenset(
    {
        "emacs_patch_policy",
        "fixture_origin",
    }
)
BRIDGE_DAEMON_NAME = "soma-gym-bridge-daemon"
BRIDGE_DAEMON_SOCKET = "/project/run/soma-gym-bridge-daemon.sock"
BRIDGE_TERMINAL_TMUX_SOCKET = "/project/run/soma-gym-bridge-terminal-tmux.sock"
BRIDGE_TERMINAL_TMUX_SESSION = "soma-gym-bridge-terminal"
BRIDGE_TERMINAL_TRANSPORT = "rootfs-owned tmux plus emacsclient --tty"


class InsulaError(RuntimeError):
    """A selected-rootfs Insula contract or invocation error."""


@dataclass(frozen=True)
class SelectedRootfs:
    store_id: str
    store_root: pathlib.Path
    rootfs: pathlib.Path
    contract_path: pathlib.Path
    contract: dict[str, Any]


@dataclass(frozen=True)
class NetworkAuthorization:
    mode: str
    task_id: str
    authorization: str
    share_net: bool


@dataclass(frozen=True)
class RunPaths:
    repo_root: pathlib.Path
    out_dir_host: pathlib.Path
    out_dir_sandbox: str
    plan_host: pathlib.Path
    plan_sandbox: str
    run_root: pathlib.Path


@dataclass(frozen=True)
class PreparedInvocation:
    selected_rootfs: SelectedRootfs
    network: NetworkAuthorization
    paths: RunPaths
    plan: dict[str, Any]
    mount_args: list[str]
    environment: dict[str, str]
    command: list[str]

    def argv(self, bwrap: str | None = None) -> list[str]:
        bwrap_bin = bwrap or shutil.which("bwrap") or "bwrap"
        env_args: list[str] = []
        for key, value in self.environment.items():
            env_args.extend(["--setenv", key, value])
        return [
            bwrap_bin,
            *self.mount_args,
            "--clearenv",
            *env_args,
            "--chdir",
            "/workspace/soma",
            *self.command,
        ]


def repo_root() -> pathlib.Path:
    return pathlib.Path(__file__).resolve().parents[2]


def load_json(path: pathlib.Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: pathlib.Path, value: dict[str, Any]) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def now_run_id() -> str:
    return f"{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}-{os.getpid()}"


def validate_run_id(run_id: str) -> str:
    if not run_id or run_id in {".", ".."} or "/" in run_id or not RUN_ID_RE.fullmatch(run_id):
        raise InsulaError(f"invalid Insula run id: {run_id!r}")
    return run_id


def resolve_selected_rootfs(store_root: pathlib.Path) -> SelectedRootfs:
    if store_root.is_symlink():
        raise InsulaError(f"rootfs store must not be a symlink: {store_root}")

    store = store_root.resolve()
    selected_path = store / "selected.json"
    if not selected_path.is_file():
        raise InsulaError(
            f"missing rootfs selection: {selected_path}; run scripts/insula/build-emacs-rootfs.sh first"
        )

    selected = load_json(selected_path)
    store_id = str(selected.get("store_id", ""))
    if not store_id or "/" in store_id or store_id in {".", ".."}:
        raise InsulaError(f"invalid selected rootfs store id: {store_id}")

    content_path = store / "content"
    if content_path.is_symlink():
        raise InsulaError(f"rootfs content directory must not be a symlink: {content_path}")

    candidate = content_path / store_id
    if candidate.is_symlink():
        raise InsulaError(f"selected rootfs must not be a symlink: {candidate}")

    rootfs = candidate.resolve()
    if rootfs.parent != content_path.resolve():
        raise InsulaError(f"selected rootfs escapes store: {rootfs}")
    if not rootfs.is_dir():
        raise InsulaError(f"selected rootfs is not a directory: {rootfs}")

    contract_path = rootfs / "etc" / "soma-insula-rootfs-contract.json"
    if not contract_path.is_file():
        raise InsulaError(f"selected rootfs lacks contract: {contract_path}")

    return SelectedRootfs(
        store_id=store_id,
        store_root=store,
        rootfs=rootfs,
        contract_path=contract_path,
        contract=load_json(contract_path),
    )


def validate_selected_rootfs(
    root: pathlib.Path,
    selected: SelectedRootfs,
    *,
    require_emacsclient: bool = False,
    run_validator: bool = True,
) -> None:
    contract_text = selected.contract_path.read_text(encoding="utf-8")
    if "/nix/store" in contract_text:
        raise InsulaError(
            "selected rootfs contract references /nix/store, which is forbidden for Soma Insula"
        )

    required_paths = [
        selected.rootfs / "opt" / "soma" / "emacs" / "bin" / "emacs",
        selected.rootfs / "opt" / "soma" / "doom-home" / ".emacs.d",
        selected.rootfs / "opt" / "soma" / "doom-home" / ".doom.d",
    ]
    if require_emacsclient:
        required_paths.append(selected.rootfs / "opt" / "soma" / "emacs" / "bin" / "emacsclient")

    for path in required_paths:
        if path.name in {"emacs", "emacsclient"}:
            if not os.access(path, os.X_OK):
                raise InsulaError(f"selected rootfs lacks executable {path}")
        elif not path.is_dir():
            raise InsulaError(f"selected rootfs lacks directory {path}")

    if run_validator:
        validator = root / "scripts" / "insula" / "validate-emacs-rootfs-contract.py"
        result = subprocess.run(
            [sys.executable, str(validator), str(selected.rootfs)],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        if result.returncode != 0:
            raise InsulaError(result.stderr.strip() or result.stdout.strip() or "rootfs contract validation failed")


def authorize_network(mode: str, task_id: str, allowed_for: str | None) -> NetworkAuthorization:
    if mode == "disabled":
        return NetworkAuthorization(
            mode=mode,
            task_id=task_id,
            authorization="disabled",
            share_net=False,
        )
    if mode in {"host", "shared", "share-net"}:
        if allowed_for != task_id:
            raise InsulaError(f"SOMA_INSULA_NETWORK={mode} requires SOMA_INSULA_ALLOW_NETWORK_FOR={task_id}")
        return NetworkAuthorization(
            mode=mode,
            task_id=task_id,
            authorization=f"SOMA_INSULA_ALLOW_NETWORK_FOR={allowed_for}",
            share_net=True,
        )
    raise InsulaError(f"unknown SOMA_INSULA_NETWORK value: {mode}")


def prepare_run_paths(root: pathlib.Path, out_dir: pathlib.Path, run_id: str) -> RunPaths:
    run_id = validate_run_id(run_id)
    root_resolved = root.resolve()
    out_abs = out_dir if out_dir.is_absolute() else root / out_dir
    out_resolved = out_abs.resolve(strict=False)
    try:
        relative_out = out_resolved.relative_to(root_resolved)
    except ValueError as exc:
        raise InsulaError(f"output directory must be under the Soma checkout: {out_resolved}") from exc

    out_resolved.mkdir(parents=True, exist_ok=True)
    run_root = root_resolved / ".scratch" / "emacs-gym" / "insula" / "runs" / run_id
    for name in ("home", "tmp", "run", "cache", "logs", "results", "eln-cache"):
        (run_root / name).mkdir(parents=True, exist_ok=True)

    out_dir_sandbox = "/workspace/soma/" + relative_out.as_posix()
    return RunPaths(
        repo_root=root_resolved,
        out_dir_host=out_resolved,
        out_dir_sandbox=out_dir_sandbox,
        plan_host=out_resolved / "insula-plan.json",
        plan_sandbox=f"{out_dir_sandbox}/insula-plan.json",
        run_root=run_root,
    )


def clean_output_dir(out_dir: pathlib.Path, patterns: Iterable[str]) -> None:
    for path in out_dir.iterdir():
        if path.is_file() and any(fnmatch.fnmatch(path.name, pattern) for pattern in patterns):
            path.unlink()


def preseed_doom_local_state(selected: SelectedRootfs, paths: RunPaths) -> None:
    local_root = selected.rootfs / "opt" / "soma" / "doom-home" / ".emacs.d" / ".local"
    for source_name, target_name in (
        ("cache", "doom-local-cache"),
        ("state", "doom-local-state"),
    ):
        target = paths.run_root / target_name
        target.mkdir(parents=True, exist_ok=True)
        source = local_root / source_name
        if source.is_dir():
            shutil.copytree(source, target, dirs_exist_ok=True, symlinks=True)


def validate_extra_fields(
    extra: dict[str, Any] | None,
    option_name: str,
    allowed_keys: frozenset[str],
) -> None:
    if not extra:
        return
    for key in sorted(extra):
        if key not in allowed_keys:
            allowed = ", ".join(sorted(allowed_keys))
            raise InsulaError(f"{option_name} {key} is not allowed; allowed keys: {allowed}")


def common_plan(
    selected: SelectedRootfs,
    paths: RunPaths,
    network: NetworkAuthorization,
    benchmark_id: str,
    *,
    process_model: str | None = None,
    worker_id: str | None = None,
    private_jupyter_runtime: bool = False,
    daemon_bridge_runtime: bool = False,
    include_doom_profile_source: bool = False,
    extra: dict[str, Any] | None = None,
) -> dict[str, Any]:
    contract = selected.contract
    emacs = contract.get("emacs", {})
    doom = contract.get("doom", {})
    tools = contract.get("tool_inventory", {})
    plan: dict[str, Any] = {
        "run_id": paths.run_root.name,
        "benchmark": benchmark_id,
        "mode": "selected-container-rootfs-insula",
        "rootfs": str(selected.rootfs),
        "rootfs_sandbox_mount": "/",
        "rootfs_contract": "/etc/soma-insula-rootfs-contract.json",
        "rootfs_contract_host_path": str(selected.contract_path),
        "rootfs_ownership": contract.get("rootfs_ownership", "soma-selected-container-rootfs"),
        "dependency_closure": contract.get("dependency_closure", "rootfs_owned"),
        "abi_guarantee": contract.get("abi_guarantee", "container-rootfs-abi-manifest"),
        "abi_manifest": "/etc/soma-insula-rootfs-contract.json",
        "abi_base": contract.get("abi_base", "container-image-rootfs"),
        "glibc_owner": contract.get("glibc_owner", "container-image-rootfs"),
        "nix_allowed": contract.get("nix_allowed", False),
        "forbidden_abi_prefixes": contract.get("forbidden_abi_prefixes", ["/nix", "/data02/home"]),
        "base_image": contract.get("base_image", ""),
        "image_tag": contract.get("image_tag", ""),
        "image_id": contract.get("image_id", ""),
        "rootfs_recipe_sha256": contract.get("rootfs_recipe_sha256", ""),
        "rootfs_tree_manifest": contract.get("rootfs_tree_manifest", {}),
        "bwrap": shutil.which("bwrap") or "bwrap",
        "emacs": "/opt/soma/emacs/bin/emacs",
        "emacs_patch_variant": "stock",
        "emacs_patch_set": [],
        "emacs_source_revision": emacs.get("source_sha256", ""),
        "emacs_source_url": emacs.get("source_url", ""),
        "emacs_configure_flags": " ".join(emacs.get("configure_flags", [])),
        "emacs_executable_hash": emacs.get("executable_sha256", ""),
        "repo_bind": "/workspace/soma",
        "repo_bind_host_path": str(paths.repo_root),
        "output_dir": paths.out_dir_sandbox,
        "doom_core_readonly": "/project/home/.emacs.d",
        "doom_user_readonly": "/project/home/.doom.d",
        "doom_core_rootfs_path": doom.get("emacsdir", ""),
        "doom_user_rootfs_path": doom.get("doomdir", ""),
        "doom_commit": doom.get("commit", ""),
        "private_home": "/project/home",
        "private_tmp": "/project/tmp",
        "private_run": "/project/run",
        "network": network.mode,
        "network_task_id": network.task_id,
        "network_authorization": network.authorization,
        "network_note": NETWORK_NOTE,
        "environment": "clearenv plus explicit allowlist",
        "production_required_tools": PRODUCTION_REQUIRED_TOOLS,
        "tool_inventory": tools,
        "visible_mount_classes": [
            "read-only selected CUDA-image rootfs mounted at /",
            "rootfs-owned Doom core/user config mounted into private HOME",
            "writable private /project runtime/cache/tmp surfaces",
            "writable Soma checkout mounted at /workspace/soma for trace output",
        ],
        "allowed_host_projections": [
            "Linux kernel",
            "host bwrap executable",
            "Soma checkout mounted at /workspace/soma",
            "run/output backing directories",
        ],
    }
    if worker_id is not None:
        plan["worker"] = worker_id
    if process_model is not None:
        plan["process_model"] = process_model
    if include_doom_profile_source:
        plan["doom_profile_source"] = doom.get("profile_source", "")
    if private_jupyter_runtime:
        plan["private_jupyter_runtime"] = "/project/run/jupyter"
    if daemon_bridge_runtime:
        plan.update(
            {
                "emacsclient": "/opt/soma/emacs/bin/emacsclient",
                "emacs_daemon_name": BRIDGE_DAEMON_NAME,
                "bridge_socket": BRIDGE_DAEMON_SOCKET,
                "daemon_ready_file": f"{paths.out_dir_sandbox}/daemon-ready.json",
                "terminal_frame_transport": BRIDGE_TERMINAL_TRANSPORT,
                "terminal_term": "xterm-256color",
                "terminal_tmux_socket": BRIDGE_TERMINAL_TMUX_SOCKET,
                "terminal_tmux_session": BRIDGE_TERMINAL_TMUX_SESSION,
            }
        )
    if extra:
        validate_extra_fields(extra, "--plan-string", EXTRA_PLAN_STRING_ALLOWLIST)
        plan.update(extra)
    return plan


def mount_args(
    selected: SelectedRootfs,
    paths: RunPaths,
    network: NetworkAuthorization,
    *,
    private_jupyter_runtime: bool = False,
    writable_doom_local_state: bool = False,
) -> list[str]:
    mounts = [
        "--unshare-all",
        "--die-with-parent",
        "--ro-bind",
        str(selected.rootfs),
        "/",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/tmp",
        "--tmpfs",
        "/run",
        "--tmpfs",
        "/project",
        "--dir",
        "/project/home",
        "--dir",
        "/project/tmp",
        "--dir",
        "/project/run",
    ]
    if private_jupyter_runtime:
        mounts.extend(["--dir", "/project/run/jupyter"])
    mounts.extend(
        [
            "--dir",
            "/project/cache",
            "--dir",
            "/project/logs",
            "--dir",
            "/project/results",
            "--ro-bind",
            str(selected.rootfs / "opt" / "soma" / "doom-home" / ".emacs.d"),
            "/project/home/.emacs.d",
            "--ro-bind",
            str(selected.rootfs / "opt" / "soma" / "doom-home" / ".doom.d"),
            "/project/home/.doom.d",
        ]
    )
    if writable_doom_local_state:
        mounts.extend(
            [
                "--bind",
                str(paths.run_root / "doom-local-cache"),
                "/project/home/.emacs.d/.local/cache",
                "--bind",
                str(paths.run_root / "doom-local-state"),
                "/project/home/.emacs.d/.local/state",
            ]
        )
    mounts.extend(
        [
            "--bind",
            str(paths.run_root / "eln-cache"),
            "/project/home/.emacs.d/eln-cache",
            "--bind",
            str(paths.repo_root),
            "/workspace/soma",
        ]
    )
    if network.share_net:
        mounts.append("--share-net")
    return mounts


def environment(
    paths: RunPaths,
    *,
    private_jupyter_runtime: bool = False,
    doom_environment: bool = False,
    daemon_bridge_runtime: bool = False,
    extra: dict[str, str] | None = None,
) -> dict[str, str]:
    env = {
        "HOME": "/project/home",
        "TMPDIR": "/project/tmp",
        "XDG_RUNTIME_DIR": "/project/run",
        "XDG_CACHE_HOME": "/project/cache",
        "XDG_CONFIG_HOME": "/project/home/.config",
        "XDG_DATA_HOME": "/project/home/.local/share",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "PATH": "/opt/soma/emacs/bin:/opt/soma/cargo/bin:/usr/local/bin:/usr/bin:/bin",
        "RUSTUP_HOME": "/opt/soma/rustup",
        "CARGO_HOME": "/opt/soma/cargo",
        "SHELL": "/bin/bash",
        "SOMA_INSULA_ACTIVE": "1",
        "SOMA_INSULA_MODE": "selected-container-rootfs",
        "SOMA_INSULA_PLAN": paths.plan_sandbox,
        "SOMA_INSULA_RUN_ID": paths.run_root.name,
    }
    if private_jupyter_runtime:
        env.update(
            {
                "JUPYTER_RUNTIME_DIR": "/project/run/jupyter",
                "IPYTHONDIR": "/project/cache/ipython",
                "MPLCONFIGDIR": "/project/cache/matplotlib",
            }
        )
    if doom_environment:
        env.update(
            {
                "EMACSDIR": "/project/home/.emacs.d",
                "DOOMDIR": "/project/home/.doom.d",
            }
        )
    if daemon_bridge_runtime:
        env.update(
            {
                "SOMA_BRIDGE_DAEMON_NAME": BRIDGE_DAEMON_NAME,
                "SOMA_BRIDGE_DAEMON_SOCKET": BRIDGE_DAEMON_SOCKET,
                "SOMA_BRIDGE_DAEMON_READY": f"{paths.out_dir_sandbox}/daemon-ready.json",
                "SOMA_BRIDGE_TERMINAL_TMUX_SOCKET": BRIDGE_TERMINAL_TMUX_SOCKET,
                "SOMA_BRIDGE_TERMINAL_TMUX_SESSION": BRIDGE_TERMINAL_TMUX_SESSION,
                "TERM": "xterm-256color",
            }
        )
    if extra:
        validate_extra_fields(extra, "--setenv", EXTRA_ENV_ALLOWLIST)
        env.update(extra)
    return env


def parse_key_value(text: str, option_name: str) -> tuple[str, str]:
    key, separator, value = text.partition("=")
    if not separator or not key:
        raise InsulaError(f"{option_name} must use KEY=VALUE syntax: {text!r}")
    return key, value


def annotate_plan_with_invocation(
    plan: dict[str, Any],
    mounts: Sequence[str],
    env: dict[str, str],
    command: Sequence[str],
) -> dict[str, Any]:
    plan["soma_insula_authority"] = "scripts/insula/soma_insula.py"
    plan["bwrap_mount_args"] = list(mounts)
    plan["environment_allowlist"] = sorted(env)
    plan["entrypoint_argv"] = list(command)
    return plan


def render_command_args(command: Sequence[str], paths: RunPaths) -> list[str]:
    substitutions = {
        "{out_dir_sandbox}": paths.out_dir_sandbox,
        "{plan_sandbox}": paths.plan_sandbox,
        "{run_id}": paths.run_root.name,
    }
    rendered = []
    for arg in command:
        value = arg
        for token, replacement in substitutions.items():
            value = value.replace(token, replacement)
        rendered.append(value)
    return rendered


def prepare_command_invocation(
    *,
    root: pathlib.Path,
    out_dir: pathlib.Path,
    benchmark_id: str,
    task_network_id: str,
    command: Sequence[str],
    store_root: pathlib.Path,
    run_id: str,
    network_mode: str,
    network_allowed_for: str | None,
    process_model: str | None = None,
    worker_id: str | None = None,
    private_jupyter_runtime: bool = False,
    daemon_bridge_runtime: bool = False,
    doom_environment: bool = False,
    writable_doom_local_state: bool = False,
    include_doom_profile_source: bool = False,
    extra_env: dict[str, str] | None = None,
    extra_plan: dict[str, Any] | None = None,
    clean_patterns: Sequence[str] = ("*.json", "*.md", "*.txt"),
    require_bwrap: bool = True,
    require_emacsclient: bool = False,
    run_validator: bool = True,
) -> PreparedInvocation:
    if not command:
        raise InsulaError("command argv must not be empty")
    if require_bwrap and shutil.which("bwrap") is None:
        raise InsulaError("bwrap is required")

    selected = resolve_selected_rootfs(store_root)
    validate_selected_rootfs(
        root,
        selected,
        require_emacsclient=require_emacsclient or daemon_bridge_runtime,
        run_validator=run_validator,
    )
    network = authorize_network(network_mode, task_network_id, network_allowed_for)
    paths = prepare_run_paths(root, out_dir, run_id)
    clean_output_dir(paths.out_dir_host, clean_patterns)
    if writable_doom_local_state:
        preseed_doom_local_state(selected, paths)

    env = environment(
        paths,
        private_jupyter_runtime=private_jupyter_runtime,
        doom_environment=doom_environment,
        daemon_bridge_runtime=daemon_bridge_runtime,
        extra=extra_env,
    )
    rendered_command = render_command_args(command, paths)
    mounts = mount_args(
        selected,
        paths,
        network,
        private_jupyter_runtime=private_jupyter_runtime,
        writable_doom_local_state=writable_doom_local_state,
    )
    plan = common_plan(
        selected,
        paths,
        network,
        benchmark_id,
        process_model=process_model,
        worker_id=worker_id,
        private_jupyter_runtime=private_jupyter_runtime,
        daemon_bridge_runtime=daemon_bridge_runtime,
        include_doom_profile_source=include_doom_profile_source,
        extra=extra_plan,
    )
    annotate_plan_with_invocation(plan, mounts, env, rendered_command)
    write_json(paths.plan_host, plan)

    return PreparedInvocation(
        selected_rootfs=selected,
        network=network,
        paths=paths,
        plan=plan,
        mount_args=mounts,
        environment=env,
        command=rendered_command,
    )


def prepare_batch_invocation(
    *,
    root: pathlib.Path,
    out_dir: pathlib.Path,
    benchmark_id: str,
    task_network_id: str,
    elisp_sandbox_path: str,
    store_root: pathlib.Path,
    run_id: str,
    network_mode: str,
    network_allowed_for: str | None,
    private_jupyter_runtime: bool = False,
    extra_env: dict[str, str] | None = None,
    extra_plan: dict[str, Any] | None = None,
    clean_patterns: Sequence[str] = ("*.json", "*.md", "*.txt"),
    require_bwrap: bool = True,
    run_validator: bool = True,
) -> PreparedInvocation:
    command = [
        "/opt/soma/emacs/bin/emacs",
        "--quick",
        "--batch",
        "-l",
        elisp_sandbox_path,
        "--",
        "{out_dir_sandbox}",
    ]
    return prepare_command_invocation(
        root=root,
        out_dir=out_dir,
        benchmark_id=benchmark_id,
        task_network_id=task_network_id,
        command=command,
        store_root=store_root,
        run_id=run_id,
        network_mode=network_mode,
        network_allowed_for=network_allowed_for,
        private_jupyter_runtime=private_jupyter_runtime,
        extra_env=extra_env,
        extra_plan=extra_plan,
        clean_patterns=clean_patterns,
        require_bwrap=require_bwrap,
        run_validator=run_validator,
    )


def exec_invocation(invocation: PreparedInvocation) -> None:
    argv = invocation.argv()
    # The selected rootfs/profile above remains Soma-specific. Namespace entry is
    # shared Insula execution; an admitted durable/runner cgroup owns descendants.
    sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[4]))
    from insula.capsule import execute_prepared
    execute_prepared(argv, env={"PATH": os.defpath})


def run_batch_command(args: argparse.Namespace) -> int:
    root = repo_root()
    store_root = pathlib.Path(
        os.environ.get(
            "SOMA_INSULA_ROOTFS_STORE",
            str(root / ".scratch" / "emacs-gym" / "insula" / "rootfs-store"),
        )
    )
    run_id = os.environ.get("SOMA_INSULA_RUN_ID", now_run_id())
    network_mode = os.environ.get("SOMA_INSULA_NETWORK", "disabled")
    network_allowed_for = os.environ.get("SOMA_INSULA_ALLOW_NETWORK_FOR", "")
    extra_env = dict(parse_key_value(item, "--setenv") for item in args.setenv)
    extra_plan = dict(parse_key_value(item, "--plan-string") for item in args.plan_string)
    invocation = prepare_batch_invocation(
        root=root,
        out_dir=pathlib.Path(args.out_dir),
        benchmark_id=args.benchmark_id,
        task_network_id=args.task_network_id,
        elisp_sandbox_path=args.elisp_sandbox_path,
        store_root=store_root,
        run_id=run_id,
        network_mode=network_mode,
        network_allowed_for=network_allowed_for,
        private_jupyter_runtime=args.private_jupyter_runtime,
        extra_env=extra_env,
        extra_plan=extra_plan,
    )
    print(
        f"\n== entering selected rootfs for {args.label} benchmark: {invocation.selected_rootfs.rootfs} ==",
        file=sys.stderr,
    )
    exec_invocation(invocation)
    raise AssertionError("execvp returned")


def run_command_command(args: argparse.Namespace) -> int:
    root = repo_root()
    store_root = pathlib.Path(
        os.environ.get(
            "SOMA_INSULA_ROOTFS_STORE",
            str(root / ".scratch" / "emacs-gym" / "insula" / "rootfs-store"),
        )
    )
    command = list(args.command_argv)
    if command and command[0] == "--":
        command = command[1:]
    if not command:
        raise InsulaError("run-command requires a command after --")

    run_id = os.environ.get("SOMA_INSULA_RUN_ID", now_run_id())
    network_mode = os.environ.get("SOMA_INSULA_NETWORK", "disabled")
    network_allowed_for = os.environ.get("SOMA_INSULA_ALLOW_NETWORK_FOR", "")
    extra_env = dict(parse_key_value(item, "--setenv") for item in args.setenv)
    extra_plan = dict(parse_key_value(item, "--plan-string") for item in args.plan_string)
    clean_patterns = tuple(args.clean_pattern) if args.clean_pattern else ("*.json", "*.md", "*.txt")
    invocation = prepare_command_invocation(
        root=root,
        out_dir=pathlib.Path(args.out_dir),
        benchmark_id=args.benchmark_id,
        task_network_id=args.task_network_id,
        command=command,
        store_root=store_root,
        run_id=run_id,
        network_mode=network_mode,
        network_allowed_for=network_allowed_for,
        process_model=args.process_model,
        worker_id=args.worker_id,
        private_jupyter_runtime=args.private_jupyter_runtime,
        daemon_bridge_runtime=args.daemon_bridge_runtime,
        doom_environment=args.doom_environment,
        writable_doom_local_state=args.writable_doom_local_state,
        include_doom_profile_source=args.include_doom_profile_source,
        extra_env=extra_env,
        extra_plan=extra_plan,
        clean_patterns=clean_patterns,
        require_emacsclient=args.require_emacsclient,
    )
    print(
        f"\n== entering selected rootfs for {args.label}: {invocation.selected_rootfs.rootfs} ==",
        file=sys.stderr,
    )
    exec_invocation(invocation)
    raise AssertionError("execvp returned")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_batch = subparsers.add_parser("run-batch")
    run_batch.add_argument("--private-jupyter-runtime", action="store_true")
    run_batch.add_argument(
        "--setenv",
        action="append",
        default=[],
        help="Add an explicit sandbox environment variable using KEY=VALUE syntax.",
    )
    run_batch.add_argument(
        "--plan-string",
        action="append",
        default=[],
        help="Add a string field to the emitted Insula plan using KEY=VALUE syntax.",
    )
    run_batch.add_argument("out_dir")
    run_batch.add_argument("benchmark_id")
    run_batch.add_argument("task_network_id")
    run_batch.add_argument("elisp_sandbox_path")
    run_batch.add_argument("label")
    run_batch.set_defaults(func=run_batch_command)

    run_command = subparsers.add_parser("run-command")
    run_command.add_argument("--private-jupyter-runtime", action="store_true")
    run_command.add_argument("--daemon-bridge-runtime", action="store_true")
    run_command.add_argument("--require-emacsclient", action="store_true")
    run_command.add_argument("--doom-environment", action="store_true")
    run_command.add_argument("--writable-doom-local-state", action="store_true")
    run_command.add_argument("--include-doom-profile-source", action="store_true")
    run_command.add_argument("--worker-id")
    run_command.add_argument("--process-model")
    run_command.add_argument(
        "--setenv",
        action="append",
        default=[],
        help="Add an explicit sandbox environment variable using KEY=VALUE syntax.",
    )
    run_command.add_argument(
        "--plan-string",
        action="append",
        default=[],
        help="Add a string field to the emitted Insula plan using KEY=VALUE syntax.",
    )
    run_command.add_argument(
        "--clean-pattern",
        action="append",
        default=[],
        help="Clean matching files from the output directory before entry.",
    )
    run_command.add_argument("out_dir")
    run_command.add_argument("benchmark_id")
    run_command.add_argument("task_network_id")
    run_command.add_argument("label")
    run_command.add_argument("command_argv", nargs=argparse.REMAINDER)
    run_command.set_defaults(func=run_command_command)

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return int(args.func(args))
    except InsulaError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
