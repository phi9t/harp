"""Unit tests for Docker-image rootfs export helpers that do not need Docker."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import docker_image_rootfs  # noqa: E402

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


class DockerImageRootfsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_safe_name_appends_snake_case_suffix(self):
        name = docker_image_rootfs.safe_name(
            "programbench/abishekvashok_1776_cmatrix.5c082c6:task_cleanroom_v6",
            "sha256:4ef6d7540041abc",
            "pytest-timeout",
        )
        self.assertEqual(
            name,
            "programbench_abishekvashok_1776_cmatrix_5c082c6_task_cleanroom_v6_4ef6d7540041_pytest_timeout",
        )

    def test_python_site_packages_prefers_usr_local(self):
        rootfs = self.root / "rootfs"
        local = rootfs / "usr/local/lib/python3.10/dist-packages"
        system = rootfs / "usr/lib/python3/dist-packages"
        system.mkdir(parents=True)
        local.mkdir(parents=True)
        self.assertEqual(docker_image_rootfs.python_site_packages(rootfs), local)

    def test_apt_package_names_are_conservative(self):
        docker_image_rootfs.validate_apt_package_names(["python3-pip", "libpq-dev", "postgresql-client-16"])
        with self.assertRaises(SystemExit):
            docker_image_rootfs.validate_apt_package_names(["python3;touch /run/corenius-tmp/nope"])

    def test_debian_archive_sources_script_is_explicit(self):
        script = docker_image_rootfs.debian_archive_sources_script()
        self.assertIn("archive.debian.org/debian", script)
        self.assertIn("archive.debian.org/debian-security", script)
        self.assertIn('Acquire::Check-Valid-Until "false";', script)

    def test_apt_customization_overrides_image_entrypoint(self):
        self.assertEqual(
            docker_image_rootfs.docker_create_command("old/image:tag", ["python3"]),
            ["docker", "create", "--entrypoint", "/bin/sh", "old/image:tag", "-c", "sleep infinity"],
        )
        self.assertEqual(
            docker_image_rootfs.docker_create_command("old/image:tag", [], ["numpy==1.26.4"]),
            ["docker", "create", "--entrypoint", "/bin/sh", "old/image:tag", "-c", "sleep infinity"],
        )
        self.assertEqual(
            docker_image_rootfs.docker_create_command("old/image:tag", []),
            ["docker", "create", "old/image:tag", "/bin/sh", "-c", "true"],
        )

    def test_create_mountpoints_validates_and_records_paths(self):
        rootfs = self.root / "rootfs"
        changes = docker_image_rootfs.create_mountpoints(
            rootfs,
            ["/root/.acme.sh"],
            ["/etc/ddclient.conf", "/etc/nginx/sites-available/default"],
        )
        self.assertTrue((rootfs / "root/.acme.sh").is_dir())
        self.assertTrue((rootfs / "etc/ddclient.conf").is_file())
        self.assertTrue((rootfs / "etc/nginx/sites-available/default").is_file())
        self.assertEqual(changes[0], {"kind": "mkdir", "path": "/root/.acme.sh"})
        with self.assertRaises(SystemExit):
            docker_image_rootfs.create_mountpoints(rootfs, ["../escape"], [])


if __name__ == "__main__":
    unittest.main()
