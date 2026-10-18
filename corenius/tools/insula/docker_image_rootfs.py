#!/usr/bin/env python3
"""Export a local Docker image into a prepared rootfs tree for insula.

This is a bridge for benchmark suites that publish task environments as OCI
images. It does not run benchmark code in Docker; it uses the Docker daemon only
to inspect/export a local image, then writes a filesystem tree that can be used
as a capsule `rootfs` layer.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
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


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_OUT_ROOT = ROOT / ".scratch/public_agent_benchmarks/rootfs"


def run(command: list[str], **kwargs) -> subprocess.CompletedProcess:
    return subprocess.run(command, text=True, check=True, **kwargs)


def inspect_image(image: str) -> dict:
    result = run(["docker", "image", "inspect", image], capture_output=True)
    docs = json.loads(result.stdout)
    if not docs:
        raise SystemExit(f"image not found locally: {image}")
    return docs[0]


def safe_name(image: str, image_id: str, name_suffix: str = "") -> str:
    stem = re.sub(r"[^a-z0-9]+", "_", image.lower()).strip("_")
    short = image_id.split(":")[-1][:12]
    suffix = re.sub(r"[^a-z0-9]+", "_", name_suffix.lower()).strip("_")
    base = f"{stem}_{short}" if stem else short
    return f"{base}_{suffix}" if suffix else base


def safe_extract_tar(stream, dest: Path) -> None:
    dest = dest.resolve()
    dirs: list[tuple[Path, int]] = []
    with tarfile.open(fileobj=stream, mode="r|") as archive:
        for member in archive:
            if member.name.startswith("/") or ".." in Path(member.name).parts:
                raise RuntimeError(f"unsafe image tar member path: {member.name}")
            if member.isdev():
                continue
            target = (dest / member.name).resolve()
            if target != dest and dest not in target.parents:
                raise RuntimeError(f"image tar member escapes rootfs: {member.name}")
            if member.isdir():
                original_mode = member.mode
                member.mode = original_mode | 0o700
                archive.extract(member, dest)
                dirs.append((target, original_mode))
            else:
                archive.extract(member, dest)
    for path, mode in reversed(dirs):
        try:
            path.chmod(mode)
        except OSError:
            pass


def python_site_packages(rootfs: Path) -> Path:
    candidates = [
        *sorted((rootfs / "usr/local/lib").glob("python*/dist-packages"), reverse=True),
        *sorted((rootfs / "usr/lib").glob("python*/dist-packages"), reverse=True),
        *sorted((rootfs / "usr/local/lib").glob("python*/site-packages"), reverse=True),
        *sorted((rootfs / "usr/lib").glob("python*/site-packages"), reverse=True),
    ]
    for candidate in candidates:
        if candidate.is_dir():
            return candidate
    raise SystemExit(f"could not find Python site-packages directory in {rootfs}")


def install_pip_packages(rootfs: Path, packages: list[str], with_deps: bool) -> list[dict]:
    if not packages:
        return []
    target = python_site_packages(rootfs)
    command = [sys.executable, "-m", "pip", "install", "--upgrade", "--target", str(target)]
    if not with_deps:
        command.append("--no-deps")
    command.extend(packages)
    subprocess.run(command, text=True, check=True)
    return [{"kind": "pip", "package": package, "target": str(target.relative_to(rootfs)), "with_deps": with_deps} for package in packages]


def install_container_pip_packages(container: str, packages: list[str], with_deps: bool) -> list[dict]:
    if not packages:
        return []
    command = [
        "docker",
        "exec",
        container,
        "python3",
        "-m",
        "pip",
        "install",
        "--upgrade",
        "--disable-pip-version-check",
    ]
    if not with_deps:
        command.append("--no-deps")
    command.extend(packages)
    run(command)
    return [{"kind": "container_pip", "package": package, "with_deps": with_deps} for package in packages]


def validate_apt_package_names(packages: list[str]) -> None:
    for package in packages:
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9+_.:-]*", package):
            raise SystemExit(f"unsafe apt package name: {package!r}")


def debian_archive_sources_script() -> str:
    return r'''
set -eu
mkdir -p /etc/apt/apt.conf.d
printf '%s\n' \
  'Acquire::Check-Valid-Until "false";' \
  'Acquire::AllowInsecureRepositories "true";' \
  'APT::Get::AllowUnauthenticated "true";' \
  > /etc/apt/apt.conf.d/99corenius-archive
for file in /etc/apt/sources.list /etc/apt/sources.list.d/*.list; do
  [ -f "$file" ] || continue
  sed -i \
    -e 's#https://deb.debian.org/debian#http://archive.debian.org/debian#g' \
    -e 's#http://deb.debian.org/debian#http://archive.debian.org/debian#g' \
    -e 's#https://security.debian.org/debian-security#http://archive.debian.org/debian-security#g' \
    -e 's#http://security.debian.org/debian-security#http://archive.debian.org/debian-security#g' \
    -e 's#https://security.debian.org#http://archive.debian.org/debian-security#g' \
    -e 's#http://security.debian.org#http://archive.debian.org/debian-security#g' \
    -e '/[[:space:]]stretch-updates[[:space:]]/s/^/# corenius-disabled /' \
    "$file"
done
'''.strip()


def configure_debian_archive_sources(container: str) -> list[dict]:
    run(["docker", "exec", container, "/bin/sh", "-c", debian_archive_sources_script()])
    return [{"kind": "apt_sources", "mode": "debian_archive"}]


def rootfs_target(rootfs: Path, value: str) -> Path:
    path = PurePosixPath(value)
    if not path.is_absolute() or ".." in path.parts or str(path) == "/":
        raise SystemExit(f"rootfs customization path must be absolute and stay inside rootfs: {value!r}")
    return rootfs / path.relative_to("/")


def create_mountpoints(rootfs: Path, dirs: list[str], files: list[str]) -> list[dict]:
    customizations = []
    for value in dirs:
        target = rootfs_target(rootfs, value)
        target.mkdir(parents=True, exist_ok=True)
        customizations.append({"kind": "mkdir", "path": value})
    for value in files:
        target = rootfs_target(rootfs, value)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.touch(exist_ok=True)
        customizations.append({"kind": "touch", "path": value})
    return customizations


def install_apt_packages(container: str, packages: list[str]) -> list[dict]:
    if not packages:
        return []
    validate_apt_package_names(packages)
    exec_base = ["docker", "exec", "-e", "DEBIAN_FRONTEND=noninteractive", container]
    run([*exec_base, "apt-get", "update"])
    run([*exec_base, "apt-get", "install", "-y", "--no-install-recommends", *packages])
    run(["docker", "exec", container, "rm", "-rf", "/var/lib/apt/lists"])
    return [{"kind": "apt", "package": package} for package in packages]


def docker_create_command(image: str, apt_packages: list[str], container_pip_packages: list[str] | None = None) -> list[str]:
    command = ["docker", "create"]
    if apt_packages or container_pip_packages:
        command += ["--entrypoint", "/bin/sh"]
    container_command = ["-c", "sleep infinity"] if apt_packages or container_pip_packages else ["/bin/sh", "-c", "true"]
    return [*command, image, *container_command]


def export_image(
    image: str,
    out_root: Path,
    replace: bool = False,
    dry_run: bool = False,
    name_suffix: str = "",
    apt_packages: list[str] | None = None,
    pip_packages: list[str] | None = None,
    pip_with_deps: bool = False,
    container_pip_packages: list[str] | None = None,
    container_pip_with_deps: bool = False,
    apt_debian_archive: bool = False,
    mkdirs: list[str] | None = None,
    touches: list[str] | None = None,
) -> Path:
    info = inspect_image(image)
    apt_packages = apt_packages or []
    pip_packages = pip_packages or []
    container_pip_packages = container_pip_packages or []
    mkdirs = mkdirs or []
    touches = touches or []
    out = out_root / safe_name(image, info["Id"], name_suffix)
    if dry_run:
        print(json.dumps({
            "image": image,
            "image_id": info["Id"],
            "out": str(out),
            "apt_packages": apt_packages,
            "apt_debian_archive": apt_debian_archive,
            "pip_packages": pip_packages,
            "container_pip_packages": container_pip_packages,
            "mkdirs": mkdirs,
            "touches": touches,
        }, indent=2))
        return out
    if out.exists():
        if not replace:
            raise SystemExit(f"rootfs already exists: {out} (pass --replace)")
        shutil.rmtree(out)
    out_root.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=f"{out.name}.", dir=str(out_root)))
    container = None
    try:
        created = run(docker_create_command(image, apt_packages, container_pip_packages), capture_output=True)
        container = created.stdout.strip()
        customizations = []
        if apt_packages or container_pip_packages:
            run(["docker", "start", container], stdout=subprocess.DEVNULL)
            if apt_debian_archive:
                customizations.extend(configure_debian_archive_sources(container))
            customizations.extend(install_apt_packages(container, apt_packages))
            customizations.extend(install_container_pip_packages(container, container_pip_packages, with_deps=container_pip_with_deps))
            subprocess.run(["docker", "stop", "--time", "1", container], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        proc = subprocess.Popen(["docker", "export", container], stdout=subprocess.PIPE)
        assert proc.stdout is not None
        safe_extract_tar(proc.stdout, staging)
        code = proc.wait()
        if code != 0:
            raise RuntimeError(f"docker export failed with exit code {code}")
        customizations.extend(install_pip_packages(staging, pip_packages, with_deps=pip_with_deps))
        customizations.extend(create_mountpoints(staging, mkdirs, touches))
        manifest = {
            "schema_version": 1,
            "image": image,
            "image_id": info["Id"],
            "repo_tags": info.get("RepoTags") or [],
            "repo_digests": info.get("RepoDigests") or [],
            "config": {
                "env": (info.get("Config") or {}).get("Env") or [],
                "working_dir": (info.get("Config") or {}).get("WorkingDir") or "",
                "entrypoint": (info.get("Config") or {}).get("Entrypoint") or [],
                "cmd": (info.get("Config") or {}).get("Cmd") or [],
            },
            "customizations": customizations,
            "exported_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        }
        (staging / "insula-rootfs-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        staging.rename(out)
        staging = None
        print(str(out))
        return out
    finally:
        if container:
            subprocess.run(["docker", "rm", "-f", container], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if staging is not None:
            shutil.rmtree(staging, ignore_errors=True)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image", help="local Docker image reference")
    parser.add_argument("--out-root", type=Path, default=DEFAULT_OUT_ROOT)
    parser.add_argument("--name-suffix", default="", help="append a snake_case suffix to the generated rootfs directory name")
    parser.add_argument("--apt-package", action="append", default=[], help="install an apt package inside the temporary Docker container before export")
    parser.add_argument("--apt-debian-archive", action="store_true", help="rewrite Debian apt sources to archive.debian.org before installing apt packages")
    parser.add_argument("--pip-package", action="append", default=[], help="install a Python package into the exported rootfs site-packages")
    parser.add_argument("--pip-with-deps", action="store_true", help="allow pip to install dependencies for --pip-package entries")
    parser.add_argument("--container-pip-package", action="append", default=[], help="install a Python package inside the temporary Docker container before export")
    parser.add_argument("--container-pip-with-deps", action="store_true", help="allow pip to install dependencies for --container-pip-package entries")
    parser.add_argument("--mkdir", action="append", default=[], help="create an absolute directory path inside the exported rootfs")
    parser.add_argument("--touch", action="append", default=[], help="create an absolute file path inside the exported rootfs")
    parser.add_argument("--replace", action="store_true")
    parser.add_argument("--dry-run", action="store_true", help="inspect and print the planned rootfs path without exporting")
    args = parser.parse_args(argv)
    export_image(
        args.image,
        args.out_root.resolve(),
        replace=args.replace,
        dry_run=args.dry_run,
        name_suffix=args.name_suffix,
        apt_packages=args.apt_package,
        pip_packages=args.pip_package,
        pip_with_deps=args.pip_with_deps,
        container_pip_packages=args.container_pip_package,
        container_pip_with_deps=args.container_pip_with_deps,
        apt_debian_archive=args.apt_debian_archive,
        mkdirs=args.mkdir,
        touches=args.touch,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
