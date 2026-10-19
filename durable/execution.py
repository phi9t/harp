"""Admitted code, persistent state and per-activity Insula projections.

The controller and guardian run outside the payload namespace. This keeps cgroup
control files private while reusing the existing Insula builder and run policy.
The binding is local admission evidence, not a signature or remote attestation.
"""
from __future__ import annotations

import copy
import os
from pathlib import Path
import shutil
from typing import Any

from insula import capsule
from durable.common import verify_input_bundle, verify_inputs_root

REPOSITORY = Path(__file__).resolve().parents[1]
PAYLOAD_ATTEMPT = "/home/bench/run"


def _verify_code_files(root: Path) -> None:
    # PYTHONDONTWRITEBYTECODE prevents writes, but Python still reads caches.
    # The existing tree digest intentionally excludes these files, so they
    # cannot be part of an admitted executable closure. The same rule applies
    # to every other exclusion: an executable can have any filename.
    for path in root.rglob("*"):
        if path.name in capsule.EXCLUDE_NAMES or path.suffix in capsule.EXCLUDE_SUFFIXES:
            raise ValueError("admitted code contains digest-excluded files (bytecode caches or .DS_Store)")


def snapshot_code(destination: Path, workflow_source: Path | None = None) -> Path:
    """Materialize the runnable Python closure, excluding tests and build output."""
    destination.mkdir(parents=True, exist_ok=False)
    for package in ("durable", "insula"):
        for source in (REPOSITORY / package).rglob("*.py"):
            if source.name.startswith("test_") or source.name.endswith("_test.py"):
                continue
            if source.is_symlink():
                raise ValueError("runtime source must not contain symlinks")
            target = destination / source.relative_to(REPOSITORY)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
    if workflow_source is not None:
        _verify_code_files(workflow_source)
        for source in workflow_source.iterdir():
            if source.name in {"durable", "insula", "__pycache__"}:
                raise ValueError("workflow source must not replace the admitted runtime")
            target = destination / source.name
            if source.is_dir():
                shutil.copytree(source, target, ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
            else:
                shutil.copy2(source, target)
    return destination


class InsulaExecution:
    def __init__(self, config: dict[str, Any]):
        self.config = copy.deepcopy(config)

    @classmethod
    def create(cls, record: dict[str, Any], code: Path, state: Path, *,
               backend: str = "process-group", cgroup_root: Path | None = None,
               inputs_root: Path | None = None,
               native_manifest: dict[str, Any] | None = None,
               providers: dict[str, Any] | None = None) -> InsulaExecution:
        if backend not in {"process-group", "cgroup"}:
            raise ValueError("unknown activity backend")
        if backend == "cgroup" and cgroup_root is None:
            raise ValueError("cgroup backend requires a delegated cgroup root")
        if backend != "cgroup" and cgroup_root is not None:
            raise ValueError("cgroup root requires cgroup backend")
        config = {"protocol": 1, "capsule": copy.deepcopy(record),
                  "capsule_store": str(capsule.store_root().resolve()),
                  "code": str(code.resolve()), "code_digest": capsule.tree_digest(code),
                  "state": str(state.resolve()), "backend": backend,
                  "cgroup_root": str(cgroup_root.resolve()) if cgroup_root else None,
                  "cgroup_identity": None}
        if inputs_root is not None:
            config["inputs_root"] = str(verify_inputs_root(inputs_root))
        if native_manifest is not None:
            config["native_manifest"] = copy.deepcopy(native_manifest)
        if providers is not None:
            config["providers"] = copy.deepcopy(providers)
        if cgroup_root is not None:
            from insula.cgroup import Cgroup
            config["cgroup_identity"] = Cgroup.validate_root(cgroup_root)
        config["contract"] = cls._contract(config)
        return cls(config)

    @staticmethod
    def _contract(config: dict[str, Any]) -> str:
        binding = {
            "capsule": config["capsule"], "capsule_store": config["capsule_store"],
            "code": config["code"], "code_digest": config["code_digest"],
            "state": config["state"], "backend": config["backend"],
            "cgroup_root": config["cgroup_root"], "cgroup_identity": config["cgroup_identity"],
        }
        for optional in ("inputs_root", "native_manifest", "providers"):
            if optional in config:
                binding[optional] = config[optional]
        return "insula-durable-v1:" + capsule.sha(capsule.canonical(binding))

    @property
    def contract(self) -> str:
        return self.config["contract"]

    @property
    def backend(self) -> str:
        return self.config["backend"]

    def verify(self, *, require_backend: bool = True) -> None:
        """Inspect retained identity; dispatch also requires the live delegated backend."""
        config = self.config
        if config["protocol"] != 1 or self._contract(config) != self.contract:
            raise ValueError("execution configuration binding changed")
        if config["backend"] not in {"process-group", "cgroup"}:
            raise ValueError("unknown activity backend")
        if str(capsule.store_root().resolve()) != config["capsule_store"]:
            raise ValueError("capsule store changed")
        _verify_code_files(Path(config["code"]))
        if capsule.tree_digest(Path(config["code"])) != config["code_digest"]:
            raise ValueError("admitted code changed")
        capsule.verify_host_layers(config["capsule"])
        for layer in config["capsule"]["layers"]:
            if layer["kind"] == "tree":
                tree = capsule.store_root() / "layers" / layer["digest"] / "tree"
                if capsule.tree_digest(tree) != layer["digest"]:
                    raise ValueError("admitted dependency tree changed")
        if "providers" in config:
            from durable.agents.provider_preflight import verify_providers
            verify_providers(config["capsule"], config["providers"])
        if "inputs_root" in config:
            verify_inputs_root(Path(config["inputs_root"]))
        if "native_manifest" in config:
            from durable.setup import verify_native_manifest
            verify_native_manifest(config["native_manifest"])
        if self.backend == "cgroup" and require_backend:
            from insula.cgroup import Cgroup
            if Cgroup.validate_root(Path(config["cgroup_root"])) != config["cgroup_identity"]:
                raise ValueError("delegated cgroup generation changed")

    def input_bundle(self, bundle_digest: str) -> Path:
        if "inputs_root" not in self.config:
            raise ValueError("activity inputs require an admitted input store")
        return verify_input_bundle(Path(self.config["inputs_root"]), bundle_digest)

    def payload(self, directory: Path, request: dict[str, Any]) -> tuple[list[str], dict[str, Any]]:
        self.verify()
        spec = request["spec"]
        code = Path(self.config["code"])
        cwd = spec["cwd"]
        if cwd is None:
            cwd = PAYLOAD_ATTEMPT
        elif cwd in {str(code), str(REPOSITORY)}:
            cwd = "/workspace"
        elif not (cwd == "/workspace" or cwd.startswith("/workspace/") or
                  cwd == PAYLOAD_ATTEMPT or cwd.startswith(PAYLOAD_ATTEMPT + "/")):
            raise ValueError("activity cwd must be in admitted code or attempt projection")
        environment = {**spec["env"], "TMPDIR": "/run/insula-tmp",
                       "TMP": "/run/insula-tmp", "TEMP": "/run/insula-tmp",
                       "TMUX_TMPDIR": "/run/insula-tmp", "TMPPREFIX": "/run/insula-tmp/zsh",
                       "PYTHONDONTWRITEBYTECODE": "1", "INSULA_DURABLE_CONTRACT": self.contract}
        inputs = self.input_bundle(spec["inputs"]) if "inputs" in spec else None
        record = copy.deepcopy(self.config["capsule"])
        provider = request.get("input", {}).get("provider") if isinstance(request.get("input"), dict) else None
        for layer in record["layers"]:
            if layer.get("name") == "provider-credentials":
                layer["binds"] = [binding for binding in layer.get("binds", [])
                                  if provider in {"codex", "trae"} and
                                  binding["dest"] == f"/home/bench/credentials/{provider}-auth.json"]
        argv = capsule.run_argv(record, str(code),
                               [(str(directory), PAYLOAD_ATTEMPT)], environment,
                               ro_workspace=True, cwd=cwd,
                               net="none" if spec.get("offline") is True else None,
                               ro=[(str(inputs), "/home/bench/input")] if inputs else None)
        # Hide cgroup controls only where the admitted layers expose them.
        # Keep declared sysfs data visible; minimal rootfs trees may omit /sys/fs
        # entirely, so they need no mask at a nonexistent readonly parent.
        masks = {"/sys/fs/cgroup"} if record["policy"]["gpu"] else set()
        projections = capsule.home_binds(record["layers"])
        for layer in record["layers"]:
            if layer["kind"] == "host-ro":
                for path in layer["paths"]:
                    projections.extend(capsule.host_binds(path))
            elif layer["kind"] == "rootfs" and (Path(layer["source"]) / "sys/fs/cgroup").exists():
                masks.add("/sys/fs/cgroup")
        for source, destination in projections:
            source, destination = os.path.realpath(source), os.path.normpath(destination)
            if source == "/sys":
                masks.add(destination + "/fs/cgroup")
            elif source == "/sys/fs":
                masks.add(destination + "/cgroup")
            elif source == "/sys/fs/cgroup" or source.startswith("/sys/fs/cgroup/"):
                if not Path(source).is_dir():
                    raise ValueError("cgroup control files cannot be projected into an activity")
                masks.add(destination)
        for destination in sorted(masks):
            argv += ["--tmpfs", destination]
        payload = copy.deepcopy(request)
        payload["attempt_dir"] = PAYLOAD_ATTEMPT
        payload.pop("execution", None)
        payload.pop("cgroup", None)
        payload["spec"]["cwd"] = cwd
        payload["spec"]["env"] = environment
        return argv, payload
