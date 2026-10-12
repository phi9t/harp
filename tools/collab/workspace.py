"""Repository and worktree operations for collab Attempts."""
from __future__ import annotations

from pathlib import Path
import subprocess

from common import Refused, child_environment


def run_git(
    root: Path,
    *args: str,
    input: str | None = None,
    check: bool = True,
    env: dict | None = None,
    raw: bool = False,
    fds: tuple[int, ...] = (),
) -> str | bytes:
    environment = child_environment()
    if env:
        environment.update(env)
    command = [
        "git",
        "-c",
        "core.hooksPath=/dev/null",
        "-c",
        "commit.gpgsign=false",
        "-c",
        "core.logAllRefUpdates=true",
        "-C",
        str(root),
        *args,
    ]
    process = subprocess.run(
        command,
        input=input,
        text=not raw,
        capture_output=True,
        env=environment,
        pass_fds=fds,
    )
    if check and process.returncode:
        raise Refused(f"Git refused {' '.join(args[:3])}: {process.stderr.strip()}")
    return process.stdout


class RepositoryWorkspace:
    def __init__(self, invoked: Path, common: Path, store, config: dict | None = None):
        self.invoked = invoked
        self.common = common
        self.store = store
        self.config = config

    def set_config(self, config: dict | None) -> None:
        self.config = config

    def require_config(self) -> dict:
        if self.config is None:
            raise Refused("Run init first; never delete state.sqlite3 to reset active work")
        return self.config

    def git(
        self,
        *args: str,
        cwd: Path | None = None,
        input: str | None = None,
        check: bool = True,
        env: dict | None = None,
        raw: bool = False,
    ) -> str | bytes:
        if cwd is None:
            self.primary_identity()
        root = cwd or Path(self.require_config()["root"])
        return run_git(
            root,
            *args,
            input=input,
            check=check,
            env=env,
            raw=raw,
            fds=tuple(self.store.fds),
        )

    def trunk(self) -> str:
        config = self.require_config()
        return self.git("rev-parse", "--verify", config["trunk_ref"] + "^{commit}").strip()

    def clean(self, path: Path, include_ignored: bool = False) -> None:
        if self.hidden_index(path):
            raise Refused(f"Workspace has hidden index entries: {path}; preserve files and classify flags manually")
        args = ["status", "--porcelain=v1", "-z", "--untracked-files=all"]
        if include_ignored:
            args.append("--ignored")
        if self.git(*args, cwd=path):
            raise Refused(f"Workspace is not clean: {path}; preserve/classify files manually")

    def hidden_index(self, path: Path) -> bool:
        # Lowercase -v tags mean assume-unchanged; S/s means skip-worktree.
        entries = self.git("ls-files", "-v", "-z", cwd=path).split("\0")
        return any(entry and (entry[0].islower() or entry[0] == "S") for entry in entries)

    def primary(self) -> None:
        self.primary_identity()
        config = self.require_config()
        root = Path(config["root"])
        ref = self.git("symbolic-ref", "-q", "HEAD", cwd=root, check=False).strip()
        if ref != config["trunk_ref"]:
            raise Refused("Primary checkout must be on configured trunk; no automatic switching")
        self.clean(root)

    def primary_identity(self) -> None:
        config = self.require_config()
        root = Path(config["root"])
        try:
            common = Path(self.git("rev-parse", "--path-format=absolute", "--git-common-dir", cwd=root).strip())
            top = Path(self.git("rev-parse", "--show-toplevel", cwd=root).strip())
            if root.resolve() != root or common.resolve() != self.common or top != root:
                raise Refused("repository or top-level mismatch")
        except (Refused, OSError) as exc:
            raise Refused(f"Primary checkout identity mismatch: {root}: {exc}") from exc

    def validate_attempt(self, attempt: dict, name: str = "path", expected_head: str | None = None) -> Path:
        """Validate identity before touching any registered attempt workspace."""
        path = Path(attempt[name])
        try:
            if path.resolve() != path or not path.is_dir():
                raise Refused("path moved or is a symlink")
            common = Path(self.git("rev-parse", "--path-format=absolute", "--git-common-dir", cwd=path).strip())
            top = Path(self.git("rev-parse", "--show-toplevel", cwd=path).strip())
            head = self.git("rev-parse", "HEAD", cwd=path).strip()
            branch = self.git("symbolic-ref", "-q", "HEAD", cwd=path, check=False).strip()
            expected_branch = "refs/heads/" + attempt["branch"] if name == "path" else ""
            records = self.git("worktree", "list", "--porcelain", "-z").split("\0\0")
            entries = [record.split("\0") for record in records if record]
            registered = next((entry for entry in entries if entry[0] == "worktree " + str(path)), [])
            valid = (
                common.resolve() == self.common
                and top == path
                and branch == expected_branch
                and "HEAD " + head in registered
                and (not expected_branch or "branch " + expected_branch in registered)
                and (expected_branch or "detached" in registered)
                and (expected_head is None or head == expected_head)
            )
            if not valid:
                raise Refused("repository, registration, branch or HEAD mismatch")
        except (Refused, OSError) as exc:
            raise Refused(f"Workspace identity mismatch: {path}: {exc}") from exc
        return path
