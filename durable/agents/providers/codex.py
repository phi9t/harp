"""One fresh Codex process inside the already admitted activity boundary.

The guardian owns timeout, cancellation and descendant cleanup. This adapter
returns completion evidence only; it cannot verify, approve or land a candidate.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess

EXECUTABLE = "/cap/providers/codex"
AUTH = Path("/home/bench/credentials/codex-auth.json")
MAX_AUTH = 1 << 20
MAX_FINAL = 256 << 10


def _read_regular(path: Path, limit: int) -> bytes:
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as stream:
        info = os.fstat(stream.fileno())
        if not stat.S_ISREG(info.st_mode) or info.st_size > limit:
            raise ValueError("Codex artifact must be a bounded regular file")
        data = stream.read(limit + 1)
    if len(data) > limit:
        raise ValueError("Codex artifact exceeds its size limit")
    return data


def _directory(path: Path, *, create: bool) -> None:
    if create:
        path.mkdir(mode=0o700)
    info = path.lstat()
    if (not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid()
            or info.st_mode & 0o077):
        raise ValueError("Codex state must be an owned private directory")


def run(workspace: Path, task: str, admission: dict, attempt: Path) -> dict:
    """Run the supplied frozen task prompt verbatim, without resuming a session."""
    attempt = Path(attempt)
    workspace = Path(workspace)
    if not attempt.is_absolute() or not workspace.is_absolute():
        raise ValueError("Codex requires absolute admitted attempt/workspace paths")
    if any(path.is_symlink() for path in (attempt, *attempt.parents, workspace, *workspace.parents)):
        raise ValueError("Codex attempt/workspace paths must not contain symlinks")
    attempt = attempt.resolve(strict=True)
    workspace = workspace.resolve(strict=True)
    if not workspace.is_dir() or workspace == attempt or not workspace.is_relative_to(attempt):
        raise ValueError("Codex workspace must belong to its admitted attempt")
    _directory(attempt, create=False)
    if not isinstance(task, str) or not task:
        raise ValueError("Codex requires the frozen task prompt")
    model = admission.get("model")
    if model is not None and (not isinstance(model, str) or not model.strip() or "\x00" in model):
        raise ValueError("Codex model must be a nonempty configured string")
    if admission.get("provider") != "codex":
        raise ValueError("Codex adapter disagrees with the admitted provider")

    auth = _read_regular(AUTH, MAX_AUTH)
    try:
        if not isinstance(json.loads(auth), dict):
            raise ValueError
    except (ValueError, UnicodeError):
        raise ValueError("Codex credential projection must contain a JSON object") from None
    home = attempt / "home"
    _directory(home, create=not home.exists())
    codex_home = home / ".codex"
    # Existing provider state is not adopted. Recovery requires a new attempt.
    _directory(codex_home, create=True)
    auth_fd = os.open(codex_home / "auth.json", os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(auth_fd, "wb") as destination:
        destination.write(auth)
    paths = {"XDG_CONFIG_HOME": home / ".config", "XDG_CACHE_HOME": home / ".cache",
             "XDG_STATE_HOME": home / ".local-state", "XDG_RUNTIME_DIR": home / ".runtime",
             "TMPDIR": attempt / "codex-tmp"}
    for path in paths.values():
        _directory(path, create=True)
    environment = {"PATH": "/cap/providers:/usr/bin:/bin", "HOME": str(home),
                   "CODEX_HOME": str(codex_home), "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8",
                   "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null",
                   "GIT_TERMINAL_PROMPT": "0", **{key: str(path) for key, path in paths.items()}}
    environment.update(TMP=environment["TMPDIR"], TEMP=environment["TMPDIR"])
    for key in ("HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY",
                "http_proxy", "https_proxy", "all_proxy", "no_proxy",
                "GIT_AUTHOR_NAME", "GIT_AUTHOR_EMAIL", "GIT_COMMITTER_NAME", "GIT_COMMITTER_EMAIL"):
        if key in os.environ:
            environment[key] = os.environ[key]
    final = attempt / "codex-final.txt"
    if final.exists() or final.is_symlink():
        raise ValueError("Codex final artifact already exists; a fresh attempt is required")
    argv = [EXECUTABLE, "--no-daemon", "--enable", "code_mode_host", "--disable", "apps",
            "--disable", "plugins", "exec", "--ignore-user-config", "--ignore-rules",
            "--ephemeral", "--dangerously-bypass-approvals-and-sandbox", "--json", "--color", "never",
            "--cd", str(workspace), "--output-last-message", str(final)]
    if model is not None:
        argv += ["--model", model]
    argv += ["-"]
    # Inherit stdout/stderr directly into guardian-owned log descriptors. There
    # is no inner timeout or detached process/session that can outlive ownership.
    result = subprocess.run(argv, input=task.encode("utf-8"), cwd=workspace, env=environment, umask=0o077)
    if result.returncode:
        raise RuntimeError(f"Codex exited with status {result.returncode}; no proposal accepted")
    text = _read_regular(final, MAX_FINAL)
    if not text or not text.decode("utf-8").strip():
        raise ValueError("Codex returned no final text")
    return {"provider": "codex", "executable": EXECUTABLE, "model": model,
            "final": {"path": final.name, "size": len(text), "sha256": hashlib.sha256(text).hexdigest()}}
