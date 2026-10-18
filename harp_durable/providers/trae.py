"""One fresh Trae coding invocation inside the admitted activity boundary."""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import stat
import subprocess
from typing import Any

from controller.durable.common import atomic_json, read_json

EXECUTABLE = Path("/opt/harp/providers/trae")
AUTH = Path("/home/bench/credentials/trae-auth.json")
MAX_FINAL_BYTES = 256 * 1024


def _private_directory(path: Path) -> None:
    if path.is_symlink():
        raise ValueError("Trae private directory must not be symlinked")
    path.mkdir(mode=0o700, exist_ok=True)
    info = path.stat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
        raise ValueError("Trae requires a private attempt home")


def run(workspace: Path, task: str, admission: dict, attempt: Path) -> dict[str, Any]:
    """Run once; the guardian owns timeout, cancellation and descendant cleanup."""
    if admission.get("provider") != "trae":
        raise ValueError("Trae adapter requires a Trae admission")
    if not isinstance(task, str) or not task.strip():
        raise ValueError("Trae task must be nonempty text")
    model = admission.get("model")
    if model is not None and (not isinstance(model, str) or not model.strip() or "\0" in model):
        raise ValueError("Trae model must be nonempty text when specified")
    workspace = workspace.resolve(strict=True)
    attempt = attempt.resolve(strict=True)
    if not workspace.is_dir() or not workspace.is_relative_to(attempt):
        raise ValueError("Trae workspace must belong to its attempt")
    home = attempt / "home"
    _private_directory(home)
    provider_home = home / ".trae"
    _private_directory(provider_home)
    cli_home = provider_home / "cli"
    _private_directory(cli_home)
    auth_path = cli_home / "auth.json"
    if auth_path.exists() or auth_path.is_symlink():
        raise ValueError("Trae attempt already has provider credentials; resume is unsupported")
    credentials = read_json(AUTH)
    if not isinstance(credentials, dict) or not credentials:
        raise ValueError("Trae credential projection must contain a JSON object")
    atomic_json(auth_path, credentials)
    final = attempt / "trae-final.txt"
    if final.exists() or final.is_symlink():
        raise ValueError("Trae final artifact already exists; resume is unsupported")
    argv = [str(EXECUTABLE), "exec", "--json", "--ephemeral", "--ignore-user-config",
            "--ignore-rules", "--disable", "daemon_auto_start", "--disable", "hooks",
            "--disable", "plugins", "-c", 'model_provider="trae"',
            "--dangerously-bypass-approvals-and-sandbox", "--cd", str(workspace),
            "--output-last-message", str(final)]
    if model is not None:
        argv += ["--model", model]
    argv.append("-")
    environment = {**os.environ, "HOME": str(home), "TRAE_HOME": str(provider_home),
                   "TRAECLI_HOME": str(cli_home), "CODEX_HOME": str(provider_home),
                   "PWD": str(workspace), "GIT_CONFIG_NOSYSTEM": "1",
                   "GIT_CONFIG_GLOBAL": "/dev/null"}
    result = subprocess.run(argv, input=task, text=True, cwd=workspace, env=environment,
                            check=False)
    if result.returncode != 0:
        raise RuntimeError(f"Trae worker exited with status {result.returncode}")
    fd = os.open(final, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as stream:
        info = os.fstat(stream.fileno())
        if not stat.S_ISREG(info.st_mode):
            raise ValueError("Trae final artifact must be an ordinary file")
        if not 0 < info.st_size <= MAX_FINAL_BYTES:
            raise ValueError("Trae final artifact must contain 1..256 KiB of UTF-8 text")
        content = stream.read(MAX_FINAL_BYTES + 1)
        if len(content) != info.st_size or len(content) > MAX_FINAL_BYTES:
            raise ValueError("Trae final artifact changed or exceeds 256 KiB")
        try:
            text = content.decode("utf-8")
        except UnicodeDecodeError as exc:
            raise ValueError("Trae final artifact must contain valid UTF-8 text") from exc
        if not text.strip():
            raise ValueError("Trae final artifact must contain nonempty text")
    return {"provider": "trae", "model": model, "final_artifact": final.name,
            "final_sha256": hashlib.sha256(content).hexdigest(), "final_size": info.st_size, "exit_code": 0}
