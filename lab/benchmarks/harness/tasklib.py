"""Benchmark tasks: load, materialize, apply the reference solution, verify inside a capsule.

Layout (Harbor-compatible authoring shape): task.toml, instruction.md, environment/ (the starting
workspace, visible to the agent), tests/ (verifier-owned, restored from here before every
verification), solution/ (solve.sh plus overlay/, never mounted for the agent).
"""

from __future__ import annotations

import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from insula import capsule  # noqa: E402

TASKS = ROOT / "lab" / "benchmarks" / "tasks"
REQUESTS = ROOT / "lab" / "benchmarks" / "insula" / "requests"
SOLUTION_COMMAND = (
    "if command -v bash >/dev/null 2>&1; then "
    "exec bash /run/insula-tmp/solution/solve.sh; "
    "else exec sh /run/insula-tmp/solution/solve.sh; fi"
)


class Task:
    def __init__(self, path: Path):
        self.path = path
        self.id = path.name
        self.cfg = tomllib.loads((path / "task.toml").read_text())
        self.bench = self.cfg.get("bench", {})
        self.verifier = self.cfg["verifier"]
        self.agent = self.cfg.get("agent", {})
        if not isinstance(self.agent, dict):
            raise ValueError("agent must be a table")
        if self.agent.get("net", "none") not in {"none", "host"}:
            raise ValueError("agent.net must be none or host")
        self.protected = self.verifier.get("protected", ["tests"])
        self.services = service_specs(self.bench.get("services", {}))
        for service in self.services:
            request = service.get("capsule_request")
            if request:
                path = Path(request)
                service["capsule_request"] = str(path if path.is_absolute() else (self.path / path).resolve())

    @property
    def tags(self) -> list[str]:
        return self.cfg.get("metadata", {}).get("tags", [])


def load_tasks(root: Path = TASKS, only: list[str] | None = None) -> list[Task]:
    tasks = [Task(p) for p in sorted(root.iterdir()) if (p / "task.toml").is_file()]
    if only:
        unknown = set(only) - {t.id for t in tasks}
        if unknown:
            raise SystemExit(f"unknown task(s): {', '.join(sorted(unknown))}")
        tasks = [t for t in tasks if t.id in only]
    return tasks


def capsule_for(task: Task) -> dict:
    return capsule.build(str(capsule_request_path(task)))


def capsule_request_path(task: Task) -> Path:
    request = task.bench.get("capsule_request")
    if request:
        path = Path(request)
        return path if path.is_absolute() else task.path / path
    return REQUESTS / f"{task.bench.get('capsule', 'toolchains')}.json"


def service_specs(raw: object) -> list[dict]:
    if not raw:
        return []
    if not isinstance(raw, dict):
        raise ValueError("bench.services must be a table")
    specs = []
    for name, cfg in sorted(raw.items()):
        if not isinstance(cfg, dict):
            raise ValueError(f"bench.services.{name} must be a table")
        command = cfg.get("command")
        if not isinstance(command, str) or not command.strip():
            raise ValueError(f"bench.services.{name}.command must be a non-empty string")
        cwd = cfg.get("cwd", "/workspace")
        ready = cfg.get("ready_command", "true")
        stop = cfg.get("stop_command", "")
        aliases = cfg.get("aliases", [])
        timeout = cfg.get("startup_timeout_sec", 10)
        phases = cfg.get("phases", ["agent", "verifier"])
        capsule_request = cfg.get("capsule_request", "")
        service_net = cfg.get("net", "none")
        service_env = cfg.get("env", {})
        ro_workspace = cfg.get("ro_workspace", False)
        if not isinstance(cwd, str) or not cwd.startswith("/"):
            raise ValueError(f"bench.services.{name}.cwd must be an absolute path")
        if not isinstance(ready, str) or not ready.strip():
            raise ValueError(f"bench.services.{name}.ready_command must be a non-empty string")
        if not isinstance(stop, str) or (stop and not stop.strip()):
            raise ValueError(f"bench.services.{name}.stop_command must be a string")
        if not isinstance(aliases, list) or any(not isinstance(alias, str) for alias in aliases):
            raise ValueError(f"bench.services.{name}.aliases must be a list of strings")
        if not isinstance(phases, list) or any(phase not in {"agent", "verifier"} for phase in phases):
            raise ValueError(f"bench.services.{name}.phases must be agent and/or verifier")
        if capsule_request and not isinstance(capsule_request, str):
            raise ValueError(f"bench.services.{name}.capsule_request must be a string")
        if service_net not in {"none", "host"}:
            raise ValueError(f"bench.services.{name}.net must be none or host")
        if not isinstance(service_env, dict) or any(not isinstance(k, str) or not isinstance(v, str) for k, v in service_env.items()):
            raise ValueError(f"bench.services.{name}.env must be a table of strings")
        if not isinstance(ro_workspace, bool):
            raise ValueError(f"bench.services.{name}.ro_workspace must be a boolean")
        specs.append({
            "name": name,
            "command": command,
            "cwd": cwd,
            "ready_command": ready,
            "stop_command": stop,
            "aliases": aliases,
            "startup_timeout_sec": float(timeout),
            "phases": phases,
            "capsule_request": capsule_request,
            "net": service_net,
            "env": service_env,
            "ro_workspace": ro_workspace,
        })
    return specs


def service_tail(path: Path, limit: int = 50) -> str:
    if not path.is_file():
        return ""
    lines = path.read_text(errors="replace").splitlines()
    return "\n".join(lines[-limit:])


def wait_for_service_ready(record: dict, ws: Path, service: dict, env: dict, net: str, gpu: bool,
                           host_aliases: list[str], lines: list[str], log_path: Path) -> bool:
    deadline = capsule.time.time() + max(1.0, float(service["startup_timeout_sec"]))
    ready_log_path = log_path.with_name(f"{log_path.stem}.ready.log")
    while True:
        with ready_log_path.open("ab") as ready_log:
            ready_log.write(f"$ {service['ready_command']}\n".encode("utf-8", "replace"))
            ready_log.flush()
            stdout = ready_log
            stderr = subprocess.STDOUT
            code = capsule.run(
                record,
                ["/bin/sh", "-c", service["ready_command"]],
                str(ws),
                [],
                env,
                net,
                gpu,
                False,
                2.0,
                service["cwd"],
                None,
                stdout=stdout,
                stderr=stderr,
                host_aliases=host_aliases,
            )
        if code == 0:
            return True
        if capsule.time.time() >= deadline:
            lines.append(f"service {service['name']} did not become ready")
            tail = service_tail(log_path)
            if tail:
                lines.append(tail)
            ready_tail = service_tail(ready_log_path)
            if ready_tail:
                lines.append(ready_tail)
            return False
        capsule.time.sleep(0.2)


def run_with_isolated_services(record: dict, ws: Path, command: str, env: dict, net: str, timeout: float,
                               services: list[dict], gpu: bool, host_aliases: list[str], on_line=None,
                               ro: list[tuple[str, str]] | None = None) -> dict:
    lines: list[str] = []

    def sink(line: str) -> None:
        lines.append(line)
        if on_line is not None:
            on_line(line)

    started = capsule.time.time()
    procs: list[tuple[subprocess.Popen, object, Path, str, dict, dict]] = []
    with tempfile.TemporaryDirectory(prefix="isolated-services-", dir=str(ROOT / ".scratch")) as tmp:
        log_root = Path(tmp)
        try:
            for service in services:
                service_record = capsule.build(service["capsule_request"])
                log_path = log_root / f"{service['name']}.log"
                handle = log_path.open("wb")
                argv = capsule.run_argv(
                    service_record,
                    str(ws),
                    [],
                    dict(service["env"]),
                    bool(service["ro_workspace"]),
                    service["net"],
                    False,
                    service["cwd"],
                    None,
                    host_aliases=service["aliases"],
                )
                proc = subprocess.Popen(
                    argv + ["--", "/bin/sh", "-c", service["command"]],
                    env={"PATH": "/usr/bin:/bin"},
                    stdout=handle,
                    stderr=subprocess.STDOUT,
                    start_new_session=True,
                )
                procs.append((proc, handle, log_path, service["name"], service_record, service))
                if not wait_for_service_ready(record, ws, service, env, net, gpu, host_aliases, lines, log_path):
                    return {"rc": 1, "timed_out": False, "seconds": round(capsule.time.time() - started, 3), "tail": "\n".join(lines[-25:])}
            code = capsule.run(
                record,
                ["/bin/sh", "-c", command],
                str(ws),
                [],
                env,
                net,
                gpu,
                False,
                timeout,
                "/workspace",
                None,
                ro,
                on_line=sink,
                host_aliases=host_aliases,
            )
            return {"rc": code, "timed_out": code == 124, "seconds": round(capsule.time.time() - started, 3), "tail": "\n".join(lines[-25:])}
        finally:
            for proc, handle, _log_path, _name, service_record, service in reversed(procs):
                if service.get("stop_command"):
                    try:
                        handle.write(f"$ {service['stop_command']}\n".encode("utf-8", "replace"))
                        handle.flush()
                        capsule.run(
                            service_record,
                            ["/bin/sh", "-c", service["stop_command"]],
                            str(ws),
                            [],
                            dict(service["env"]),
                            service["net"],
                            False,
                            bool(service["ro_workspace"]),
                            5.0,
                            service["cwd"],
                            None,
                            stdout=handle,
                            stderr=subprocess.STDOUT,
                            host_aliases=service["aliases"],
                        )
                    except Exception as exc:
                        handle.write(f"stop_command failed: {exc}\n".encode("utf-8", "replace"))
                        handle.flush()
                try:
                    os.killpg(os.getpgid(proc.pid), 15)
                except OSError:
                    pass
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    try:
                        os.killpg(os.getpgid(proc.pid), 9)
                    except OSError:
                        pass
                    proc.wait(timeout=5)
                handle.close()


def wrap_with_services(command: str, services: list[dict] | None) -> str:
    services = services or []
    if not services:
        return command
    lines = [
        "set -eu",
        "mkdir -p /run/insula-tmp/services",
        "pids=''",
        "cleanup() {",
        "  rc=$?",
        "  set +e",
        "  for pid in $pids; do kill \"$pid\" 2>/dev/null || true; done",
        "  for pid in $pids; do wait \"$pid\" 2>/dev/null || true; done",
        "  exit \"$rc\"",
        "}",
        "trap cleanup EXIT HUP INT TERM",
    ]
    stop_lines: list[str] = []
    for service in services:
        name = shlex.quote(service["name"])
        cwd = shlex.quote(service["cwd"])
        service_command = service["command"]
        ready_command = service["ready_command"]
        stop_command = service.get("stop_command", "")
        timeout = max(1, int(service["startup_timeout_sec"]))
        if stop_command:
            stop_lines.append(f"( cd {cwd} && {stop_command} ) >/run/insula-tmp/services/{name}.stop.log 2>&1 || true")
        lines.extend([
            f"( cd {cwd} && {service_command} ) > /run/insula-tmp/services/{name}.log 2>&1 &",
            "pid=$!",
            "pids=\"$pids $pid\"",
            f"deadline=$(( $(date +%s) + {timeout} ))",
            "while :; do",
            "  if ! kill -0 \"$pid\" 2>/dev/null; then",
            f"    echo service {service['name']} exited before readiness >&2",
            f"    tail -n 50 /run/insula-tmp/services/{name}.log >&2 || true",
            "    exit 1",
            "  fi",
            f"  if ( cd {cwd} && {ready_command} ) >/run/insula-tmp/services/{name}.ready.log 2>&1; then break; fi",
            "  if [ \"$(date +%s)\" -ge \"$deadline\" ]; then",
            f"    echo service {service['name']} did not become ready >&2",
            f"    tail -n 50 /run/insula-tmp/services/{name}.log >&2 || true",
            f"    tail -n 50 /run/insula-tmp/services/{name}.ready.log >&2 || true",
            "    exit 1",
            "  fi",
            "  sleep 0.2",
            "done",
        ])
    lines[6:6] = stop_lines
    lines.extend([
        "set +e",
        "(",
        command,
        ")",
        "rc=$?",
        "exit \"$rc\"",
    ])
    return "\n".join(lines)


def copy_file_preserving_execute_only(source: str | Path, target: str | Path) -> None:
    source = Path(source)
    target = Path(target)
    mode = source.stat().st_mode & 0o7777
    restored = False
    if not os.access(source, os.R_OK):
        source.chmod(mode | 0o400)
        restored = True
    try:
        shutil.copy2(source, target)
    finally:
        if restored:
            source.chmod(mode)
    target.chmod(mode)


def copy_tree_preserving_restricted_modes(source: Path, target: Path) -> None:
    """Copy a task environment while preserving deliberately restrictive modes."""
    if source.is_symlink():
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists() or target.is_symlink():
            target.unlink()
        os.symlink(os.readlink(source), target)
        return
    if source.is_file():
        copy_file_preserving_execute_only(source, target)
        return
    if not source.is_dir():
        return

    mode = source.stat().st_mode & 0o7777
    restored = False
    if not os.access(source, os.R_OK | os.X_OK):
        source.chmod(mode | 0o500)
        restored = True
    try:
        target.mkdir(parents=True, exist_ok=True)
        for child in sorted(source.iterdir()):
            copy_tree_preserving_restricted_modes(child, target / child.name)
    finally:
        if restored:
            source.chmod(mode)
    target.chmod(mode)


def materialize(task: Task, ws: Path) -> None:
    copy_tree_preserving_restricted_modes(task.path / "environment", ws)


def restore_protected(task: Task, ws: Path) -> None:
    """Replace verifier-owned paths from the pristine task copy; the agent's edits there are discarded."""
    for rel in task.protected:
        if ".." in Path(rel).parts or Path(rel).is_absolute():
            raise ValueError(f"protected path {rel} escapes the workspace")
        target = ws / rel
        if target.is_symlink() or target.is_file():
            target.unlink()
        elif target.is_dir():
            shutil.rmtree(target)
        source = task.path / rel
        if not source.exists():
            source = task.path / "environment" / rel  # protected source files come from the starting workspace
        if source.is_dir():
            shutil.copytree(source, target, symlinks=False)
            for path in target.rglob("*"):
                if path.is_symlink():
                    raise ValueError(f"symlink in pristine {rel}: {path}")
        elif source.is_file():
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)


def run_in_capsule(record: dict, ws: Path, command: str, env: dict, net: str, timeout: float,
                   ro: list[tuple[str, str]] | None = None, gpu: bool = False, on_line=None,
                   services: list[dict] | None = None) -> dict:
    """Run `command` in the capsule. With on_line, every output line is also passed to the callback as it arrives."""
    lines: list[str] = []

    def sink(line: str) -> None:
        lines.append(line)
        if on_line is not None:
            on_line(line)

    started = capsule.time.time()
    same_capsule_services = [service for service in services or [] if not service.get("capsule_request")]
    isolated_services = [service for service in services or [] if service.get("capsule_request")]
    host_aliases = sorted({alias for service in services or [] for alias in service["aliases"]})
    if isolated_services:
        return run_with_isolated_services(
            record,
            ws,
            wrap_with_services(command, same_capsule_services),
            env,
            net,
            timeout,
            isolated_services,
            gpu,
            host_aliases,
            on_line=on_line,
            ro=ro,
        )
    code = capsule.run(record, ["/bin/sh", "-c", wrap_with_services(command, same_capsule_services)], str(ws), [], env, net, gpu, False, timeout,
                       "/workspace", None, ro, on_line=sink, host_aliases=host_aliases)
    elapsed = capsule.time.time() - started
    return {"rc": code, "timed_out": code == 124, "seconds": round(elapsed, 3), "tail": "\n".join(lines[-25:])}


def verify(task: Task, ws: Path, record: dict) -> dict:
    restore_protected(task, ws)
    services = [service for service in task.services if "verifier" in service["phases"]]
    result = run_in_capsule(record, ws, task.verifier["command"], dict(task.verifier.get("env", {})),
                            task.bench.get("net", "none"), float(task.verifier.get("timeout_sec", 300)),
                            gpu=bool(task.bench.get("gpu", False)), services=services)
    result["passed"] = result["rc"] == 0
    return result


def apply_solution(task: Task, ws: Path, record: dict) -> dict:
    solution = task.path / "solution"
    services = [service for service in task.services if "agent" in service["phases"]]
    return run_in_capsule(record, ws, SOLUTION_COMMAND, dict(task.agent.get("env", {})),
                          task.agent.get("net", "none"), 120.0,
                          ro=[(str(solution), "/run/insula-tmp/solution")], services=services)
