"""capsule.py against real bwrap in a throwaway store (no network, no GPU needed)."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import capsule  # noqa: E402

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "insula" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "insula" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


@unittest.skipUnless(sys.platform == "linux", "native Linux capsule qualification")
class CapsuleTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        os.environ["INSULA_CAPSULES"] = str(self.root / "store")
        self.tool = self.root / "tool"
        (self.tool / "bin").mkdir(parents=True)
        (self.tool / "bin" / "hello").write_text("#!/bin/sh\necho hello-from-layer\n")
        (self.tool / "bin" / "hello").chmod(0o755)
        self.hostdir = self.root / "hostro"
        self.hostdir.mkdir()
        (self.hostdir / "f").write_text("host\n")
        self.ws = self.root / "ws"
        self.ws.mkdir()

    def busybox_rootfs(self):
        rootfs = self.root / "rootfs"
        (rootfs / "usr/bin").mkdir(parents=True)
        (rootfs / "lib/x86_64-linux-gnu").mkdir(parents=True)
        (rootfs / "lib64").mkdir()
        shutil.copy2("/usr/bin/busybox", rootfs / "usr/bin/busybox")
        for applet in ("cat", "grep", "id", "mkdir", "sh"):
            (rootfs / "usr/bin" / applet).symlink_to("busybox")
        (rootfs / "bin").symlink_to("usr/bin")
        for src in ("/lib/x86_64-linux-gnu/libresolv.so.2", "/lib/x86_64-linux-gnu/libc.so.6"):
            shutil.copy2(src, rootfs / src.lstrip("/"))
        shutil.copy2("/lib64/ld-linux-x86-64.so.2", rootfs / "lib64/ld-linux-x86-64.so.2")
        return rootfs

    def tearDown(self):
        os.environ.pop("INSULA_CAPSULES", None)
        self.tmp.cleanup()

    def request(self, **over):
        req = {
            "name": "t",
            "layers": [
                {"kind": "base", "source": "host-usr"},
                {"kind": "tree", "name": "tool", "source": str(self.tool), "path": ["bin"]},
                {"kind": "host-ro", "name": "hostro", "paths": [str(self.hostdir)]},
            ],
            "policy": {"net": "none"},
        }
        req.update(over)
        path = self.root / "req.json"
        path.write_text(json.dumps(req))
        return str(path)

    def sh(self, record, script, **kw):
        out = self.root / "out.txt"
        with out.open("w") as handle:
            argv = capsule.run_argv(
                record,
                str(self.ws),
                kw.get("rw", []),
                kw.get("env", {}),
                False,
                kw.get("net"),
                host_aliases=kw.get("host_aliases"),
            )
            proc = subprocess.run(argv + ["--", "/bin/sh", "-c", script], stdout=handle, stderr=subprocess.STDOUT,
                                  env={"PATH": "/usr/bin:/bin"})
        return proc.returncode, out.read_text()

    def test_build_is_deterministic_and_layer_change_moves_only_that_layer(self):
        first = capsule.build(self.request())
        second = capsule.build(self.request())
        self.assertEqual(first["capsule_digest"], second["capsule_digest"])
        (self.tool / "bin" / "hello").write_text("#!/bin/sh\necho changed\n")
        third = capsule.build(self.request())
        self.assertNotEqual(first["capsule_digest"], third["capsule_digest"])
        moved = [a["name"] for a, b in zip(first["layers"], third["layers"]) if a["digest"] != b["digest"]]
        self.assertEqual(moved, ["tool"])

    def test_layer_runs_and_workspace_is_writable(self):
        record = capsule.build(self.request())
        code, out = self.sh(record, "hello; echo x > /workspace/made; cat /cap/tree/tool/bin/hello | head -1")
        self.assertEqual(code, 0, out)
        self.assertIn("hello-from-layer", out)
        self.assertEqual((self.ws / "made").read_text(), "x\n")

    def test_prepared_rootfs_can_replace_the_host_usr_base(self):
        layers = [{"kind": "rootfs", "name": "busybox", "source": str(self.busybox_rootfs())}]
        record = capsule.build(self.request(layers=layers))
        code, out = self.sh(record, "echo rootfs > /workspace/from-rootfs; cat /workspace/from-rootfs; test ! -e /cap/tree/tool")
        self.assertEqual((code, out), (0, "rootfs\n"))
        self.assertEqual((self.ws / "from-rootfs").read_text(), "rootfs\n")

    def test_prepared_rootfs_has_current_user_identity(self):
        layers = [{"kind": "rootfs", "name": "busybox", "source": str(self.busybox_rootfs())}]
        record = capsule.build(self.request(layers=layers))
        code, out = self.sh(record, "grep \"^bench:x:$(id -u):$(id -g):\" /etc/passwd; grep \"^bench:x:$(id -g):\" /etc/group")
        self.assertEqual(code, 0, out)

    def test_prepared_rootfs_preserves_package_identities(self):
        rootfs = self.busybox_rootfs()
        (rootfs / "etc").mkdir()
        (rootfs / "etc/passwd").write_text(
            "root:x:0:0:root:/root:/bin/sh\n"
            "www-data:x:33:33:www-data:/var/www:/usr/sbin/nologin\n"
        )
        (rootfs / "etc/group").write_text(
            "root:x:0:\n"
            "www-data:x:33:\n"
        )
        layers = [{"kind": "rootfs", "name": "busybox", "source": str(rootfs)}]
        record = capsule.build(self.request(layers=layers))
        code, out = self.sh(record, "grep '^www-data:x:33:33:' /etc/passwd; grep '^www-data:x:33:' /etc/group")
        self.assertEqual(code, 0, out)

    def test_prepared_rootfs_can_have_declared_runtime_identities(self):
        try:
            capsule.select_subid_range("uid", 2003)
            capsule.select_subid_range("gid", 2003)
        except capsule.CapsuleError as exc:
            self.skipTest(str(exc))
        layers = [{"kind": "rootfs", "name": "busybox", "source": str(self.busybox_rootfs())}]
        record = capsule.build(self.request(layers=layers, policy={
            "net": "none",
            "user": "root",
            "identity": {
                "groups": [
                    {"name": "research", "gid": 2000, "members": ["alice", "bob"]},
                    {"name": "alice", "gid": 2001, "members": []},
                    {"name": "bob", "gid": 2002, "members": []},
                    {"name": "mallory", "gid": 2003, "members": []},
                ],
                "users": [
                    {"name": "alice", "uid": 2001, "gid": 2001, "home": "/home/alice", "shell": "/bin/sh"},
                    {"name": "bob", "uid": 2002, "gid": 2002, "home": "/home/bob", "shell": "/bin/sh"},
                    {"name": "mallory", "uid": 2003, "gid": 2003, "home": "/home/mallory", "shell": "/bin/sh"},
                ],
            },
        }))
        code, out = self.sh(record, "cat /proc/self/uid_map; cat /proc/self/gid_map; "
                            "grep '^alice:x:2001:2001:' /etc/passwd; "
                            "grep '^research:x:2000:alice,bob' /etc/group")
        self.assertEqual(code, 0, out)
        self.assertIn(" 1 ", out)
        self.assertIn(" 2003", out)

    def test_identity_requires_prepared_rootfs(self):
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(policy={
                "net": "none",
                "user": "root",
                "identity": {"users": [{"name": "alice", "uid": 2001, "gid": 2001}], "groups": []},
            }))

    def test_prepared_rootfs_can_receive_runtime_hosts_aliases(self):
        layers = [{"kind": "rootfs", "name": "busybox", "source": str(self.busybox_rootfs())}]
        record = capsule.build(self.request(layers=layers))
        code, out = self.sh(record, "grep '^127.0.0.1 db$' /etc/hosts", host_aliases=["db"])
        self.assertEqual(code, 0, out)

    def test_prepared_rootfs_has_localhost_runtime_hosts(self):
        layers = [{"kind": "rootfs", "name": "busybox", "source": str(self.busybox_rootfs())}]
        record = capsule.build(self.request(layers=layers))
        code, out = self.sh(record, "grep '^127.0.0.1 localhost$' /etc/hosts")
        self.assertEqual(code, 0, out)

    def test_host_net_capsule_uses_runtime_hosts(self):
        record = capsule.build(self.request())
        code, out = self.sh(
            record,
            "grep '^127.0.0.1 localhost$' /etc/hosts; grep '^127.0.0.1 api$' /etc/hosts",
            net="host",
            host_aliases=["api"],
        )
        self.assertEqual(code, 0, out)

    def test_policy_can_grant_net_bind_service_to_root_user(self):
        record = capsule.build(self.request(policy={
            "net": "none",
            "user": "root",
            "cap_add": ["CAP_NET_BIND_SERVICE"],
        }))
        code, out = self.sh(record, "id -u; python3 - <<'PY'\n"
                            "import socket\n"
                            "sock = socket.socket()\n"
                            "sock.bind(('127.0.0.1', 443))\n"
                            "print('bound')\n"
                            "PY")
        self.assertEqual(code, 0, out)
        self.assertIn("0\nbound\n", out)

    def test_policy_cap_add_is_validated(self):
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(policy={"net": "none", "cap_add": ["CAP_NET_BIND_SERVICE"]}))
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(policy={"net": "none", "user": "root", "cap_add": ["CAP_SYS_ADMIN"]}))

    def test_prepared_rootfs_does_not_shadow_workspace_aliases(self):
        rootfs = self.busybox_rootfs()
        (rootfs / "app").mkdir()
        (rootfs / "logs/artifacts").mkdir(parents=True)
        (rootfs / "app/from-image").write_text("image\n")
        layers = [{"kind": "rootfs", "name": "busybox", "source": str(rootfs)}]
        record = capsule.build(self.request(layers=layers, policy={
            "net": "none",
            "workspace_aliases": ["/app"],
            "workspace_path_aliases": {"/logs": "logs"},
        }))
        code, out = self.sh(record, "echo workspace > /app/from-workspace; "
                            "mkdir -p /logs/artifacts; "
                            "echo log > /logs/artifacts/out; "
                            "cat /workspace/from-workspace /workspace/logs/artifacts/out; "
                            "test ! -e /app/from-image")
        self.assertEqual((code, out), (0, "workspace\nlog\n"))
        self.assertEqual((self.ws / "from-workspace").read_text(), "workspace\n")

    def test_usr_and_layers_are_read_only_and_home_is_hidden(self):
        record = capsule.build(self.request())
        home = str(Path.home())
        code, out = self.sh(record, f"touch /usr/x 2>&1; touch /cap/tree/tool/x 2>&1; ls -A {home} 2>&1; ls /home")
        self.assertEqual(out.count("Read-only"), 2, out)
        listed = out.split("system\n", 2)[-1].split()
        self.assertNotIn(".ssh", listed)
        self.assertNotIn("workspace", listed)
        self.assertEqual(listed[-1], "bench")

    def test_offline_has_no_interfaces_and_host_mode_keeps_them(self):
        record = capsule.build(self.request())
        iface = "tail -n +3 /proc/net/dev | cut -d: -f1 | tr -d ' '"
        _, offline = self.sh(record, iface)
        self.assertEqual(offline.split(), ["lo"])
        _, online = self.sh(record, iface, net="host")
        self.assertGreater(len(online.split()), 1)

    def test_env_is_an_allowlist_and_forbidden_vars_are_refused(self):
        os.environ["LIBRARY_PATH"] = "/nowhere"
        os.environ["SECRET_TOKEN"] = "s3cr3t"
        try:
            record = capsule.build(self.request())
            code, out = self.sh(record, "env")
        finally:
            os.environ.pop("LIBRARY_PATH")
            os.environ.pop("SECRET_TOKEN")
        self.assertNotIn("LIBRARY_PATH", out)
        self.assertNotIn("s3cr3t", out)
        self.assertIn("INSULA_IN_INSULA=1", out)
        self.assertIn("TMPDIR=/run/insula-tmp", out)
        self.assertIn("TMUX_TMPDIR=/run/insula-tmp", out)
        self.assertIn("TMPPREFIX=/run/insula-tmp/zsh", out)
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(env={"LIBRARY_PATH": "/x"}))

    def test_shadowing_and_reserved_paths_are_refused(self):
        layers = [{"kind": "base"}, {"kind": "host-ro", "name": "a", "paths": ["/opt/x"]},
                  {"kind": "host-ro", "name": "b", "paths": ["/opt/x/y"]}]
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(layers=layers))
        layers = [{"kind": "base"}, {"kind": "host-ro", "name": "a", "paths": ["/etc/ssl"]}]
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(layers=layers))

    def test_first_layer_must_be_base_and_kinds_are_checked(self):
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(layers=[{"kind": "tree", "name": "t", "source": str(self.tool)}]))
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(layers=[{"kind": "base"}, {"kind": "nope", "name": "x"}]))

    def test_stale_host_layer_is_refused(self):
        record = capsule.build(self.request())
        (self.hostdir / "g").write_text("new file changes the fingerprint\n")
        with self.assertRaises(capsule.CapsuleError):
            capsule.verify_host_layers(record)

    def test_tampered_tree_layer_fails_the_identity_probe(self):
        record = capsule.build(self.request())
        layer = next(l for l in record["layers"] if l["name"] == "tool")
        target = capsule.store_root() / "layers" / layer["digest"] / "tree" / "bin" / "hello"
        target.chmod(0o755)
        target.write_text("tampered\n")
        result = capsule.probe(record)
        self.assertTrue(result["identity"].startswith("FAIL"))
        self.assertFalse(result["ok"])

    def test_tree_layer_can_stage_regular_files_as_hardlinks(self):
        record = capsule.build(self.request(layers=[
            {"kind": "base"},
            {"kind": "tree", "name": "tool", "source": str(self.tool), "link_mode": "hardlink"},
        ]))
        layer = next(l for l in record["layers"] if l["name"] == "tool")
        target = capsule.store_root() / "layers" / layer["digest"] / "tree" / "bin" / "hello"
        self.assertEqual(os.stat(self.tool / "bin" / "hello").st_ino, os.stat(target).st_ino)

    def test_probe_passes_on_a_clean_capsule(self):
        result = capsule.probe(capsule.build(self.request()))
        failing = {k: v for k, v in result.items() if isinstance(v, str) and v.startswith("FAIL")}
        self.assertEqual(failing, {}, result)
        self.assertEqual(result["offline"], "PASS")
        self.assertIn("hostro", result["hermetic"])

    def test_rw_destination_must_be_inside_allowed_roots(self):
        record = capsule.build(self.request())
        with self.assertRaises(capsule.CapsuleError):
            capsule.run_argv(record, str(self.ws), [(str(self.ws), "/usr/evil")], {}, False)
        (self.root / "cache").mkdir()
        code, out = self.sh(record, "echo c > /home/bench/cache/f", rw=[(str(self.root / "cache"), "/home/bench/cache")])
        self.assertEqual(code, 0, out)
        self.assertEqual((self.root / "cache" / "f").read_text(), "c\n")

    def test_workspace_alias_points_at_workspace(self):
        record = capsule.build(self.request(policy={
            "net": "none",
            "workspace_aliases": ["/app"],
            "workspace_path_aliases": {"/tests": "tests"},
        }))
        (self.ws / "tests").mkdir()
        code, out = self.sh(record, "echo alias > /app/from-alias; echo test > /tests/from-tests; "
                            "cat /workspace/from-alias /workspace/tests/from-tests")
        self.assertEqual((code, out), (0, "alias\ntest\n"))
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(policy={"net": "none", "workspace_aliases": ["/usr"]}))
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(policy={"net": "none", "workspace_path_aliases": {"/tests": "../escape"}}))

    def test_workspace_writable_overlays_bind_dirs_and_files(self):
        record = capsule.build(self.request(policy={
            "net": "none",
            "workspace_writable_overlays": [
                {"dest": "/etc/corenius.d", "path": ".insula_overlays/etc/corenius.d", "kind": "dir"},
                {"dest": "/etc/corenius.conf", "path": ".insula_overlays/etc/corenius.conf", "kind": "file"},
            ],
        }))
        code, out = self.sh(record, "echo dir > /etc/corenius.d/value; "
                            "echo file > /etc/corenius.conf; "
                            "cat /etc/corenius.d/value /etc/corenius.conf")
        self.assertEqual((code, out), (0, "dir\nfile\n"))
        self.assertEqual((self.ws / ".insula_overlays/etc/corenius.d/value").read_text(), "dir\n")
        self.assertEqual((self.ws / ".insula_overlays/etc/corenius.conf").read_text(), "file\n")

    def test_rootfs_workspace_writable_overlay_can_replace_root_path(self):
        rootfs = self.busybox_rootfs()
        (rootfs / "srv").mkdir()
        (rootfs / "srv/image").write_text("image\n")
        record = capsule.build(self.request(layers=[
            {"kind": "rootfs", "name": "busybox", "source": str(rootfs)}
        ], policy={
            "net": "none",
            "workspace_writable_overlays": [
                {"dest": "/srv", "path": ".insula_overlays/srv", "kind": "dir"},
            ],
        }))
        code, out = self.sh(record, "mkdir -p /srv/shared; echo value > /srv/shared/file; "
                            "test ! -e /srv/image; cat /srv/shared/file")
        self.assertEqual((code, out), (0, "value\n"))
        self.assertEqual((self.ws / ".insula_overlays/srv/shared/file").read_text(), "value\n")

    def test_workspace_writable_overlays_are_validated(self):
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(policy={
                "net": "none",
                "workspace_writable_overlays": [{"dest": "/workspace/evil", "path": "evil", "kind": "dir"}],
            }))
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(policy={
                "net": "none",
                "workspace_writable_overlays": [{"dest": "/etc/x", "path": "../escape", "kind": "dir"}],
            }))
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(policy={
                "net": "none",
                "workspace_writable_overlays": [
                    {"dest": "/etc/app", "path": "app", "kind": "dir"},
                    {"dest": "/etc/app/conf", "path": "conf", "kind": "dir"},
                ],
            }))

    def test_timeout_kills_and_record_is_written(self):
        record = capsule.build(self.request())
        rec = self.root / "rec.json"
        code = capsule.run(record, ["/bin/sh", "-c", "sleep 30"], str(self.ws), [], {}, None, None, False, 1.0,
                           "/workspace", str(rec))
        self.assertEqual(code, 124)
        data = json.loads(rec.read_text())
        self.assertTrue(data["timed_out"])
        self.assertEqual(data["capsule_digest"], record["capsule_digest"])

    def test_gc_removes_only_unreferenced_layers(self):
        first = capsule.build(self.request())
        (self.tool / "bin" / "hello").write_text("#!/bin/sh\necho v2\n")
        second = capsule.build(self.request())
        old = next(l["digest"] for l in first["layers"] if l["name"] == "tool")
        (capsule.store_root() / "capsules" / f"{first['capsule_digest']}.json").unlink()
        removed = capsule.gc()
        self.assertIn(old, removed)
        kept = next(l["digest"] for l in second["layers"] if l["name"] == "tool")
        self.assertTrue((capsule.store_root() / "layers" / kept).exists())

    @unittest.skipUnless(Path("/dev/nvidia0").exists(), "no GPU on this host")
    def test_gpu_mode_exposes_devices_and_default_does_not(self):
        record = capsule.build(self.request())
        _, without = self.sh(record, "ls /dev | grep -c '^nvidia' || true")
        self.assertEqual(without.strip(), "0")
        argv = capsule.run_argv(record, str(self.ws), [], {}, False, gpu=True)
        proc = subprocess.run(argv + ["--", "/bin/sh", "-c", "ls /dev | grep -c '^nvidia'; test -d /sys/bus/pci && echo sys"],
                              capture_output=True, text=True, env={"PATH": "/usr/bin:/bin"})
        self.assertGreater(int(proc.stdout.split()[0]), 1, proc.stderr)
        self.assertIn("sys", proc.stdout)

    def test_symlinked_host_path_is_followed(self):
        real = self.root / "real"
        real.mkdir()
        (real / "marker").write_text("m\n")
        link = self.root / "link"
        link.symlink_to(real)
        layers = [{"kind": "base"}, {"kind": "host-ro", "name": "l", "paths": [str(link)]}]
        record = capsule.build(self.request(layers=layers))
        code, out = self.sh(record, f"cat {link}/marker")
        self.assertEqual((code, out), (0, "m\n"))

    def test_home_file_binds_are_read_only_and_never_hashed_by_content(self):
        secret = self.root / "cfg.toml"
        secret.write_text("token = 'not-a-real-secret'\n")
        layers = [{"kind": "base"}, {"kind": "host-ro", "name": "cfg",
                                     "binds": [{"src": str(secret), "dest": "/home/bench/.app/cfg.toml"}]}]
        record = capsule.build(self.request(layers=layers))
        code, out = self.sh(record, "cat /home/bench/.app/cfg.toml; echo x > /home/bench/.app/cfg.toml 2>&1; ls /home/bench")
        self.assertIn("not-a-real-secret", out)
        self.assertIn("Read-only", out)
        self.assertNotIn("not-a-real-secret", json.dumps(record))  # digest is stat based, record holds no contents
        secret.write_text("token = 'changed-and-longer'\n")
        with self.assertRaises(capsule.CapsuleError):
            capsule.verify_host_layers(record)  # a changed credential file makes the capsule stale

    def test_bind_dest_outside_home_is_refused(self):
        layers = [{"kind": "base"}, {"kind": "host-ro", "name": "x", "binds": [{"src": "/etc/hostname", "dest": "/usr/x"}]}]
        with self.assertRaises(capsule.CapsuleError):
            capsule.build(self.request(layers=layers))

    def test_extends_merges_layers_policy_and_env(self):
        parent = self.root / "parent.json"
        parent.write_text(json.dumps({"name": "p", "layers": [{"kind": "base"}], "policy": {"net": "none"}, "env": {"A": "1"}}))
        child = self.root / "child.json"
        child.write_text(json.dumps({"name": "c", "extends": "parent.json",
                                     "layers": [{"kind": "host-ro", "name": "h", "paths": [str(self.hostdir)]}],
                                     "policy": {"net": "host"}, "env": {"B": "2"}}))
        record = capsule.build(str(child))
        self.assertEqual([l["name"] for l in record["layers"]], ["base", "h"])
        self.assertEqual((record["policy"]["net"], record["env"]), ("host", {"A": "1", "B": "2"}))
        code, out = self.sh(record, "echo $A$B")
        self.assertEqual(out.strip(), "12")

    def test_real_toolchains_request_does_not_expose_the_cargo_home_or_other_agents_config(self):
        request = Path(capsule.__file__).resolve().parents[1] / "lab/benchmarks/insula/requests/agent.json"
        if not request.exists():
            self.skipTest("no agent request")
        record = capsule.build(str(request))
        code, out = self.sh(record, "ls -A ~/.cargo 2>&1; ls -A /home/bench/.cargo 2>&1; ls -A " +
                            str(Path.home() / ".cargo") + " 2>&1")
        for hidden in (".codex", ".trae", ".agents", ".git", "registry", "credentials.toml"):
            self.assertNotIn(hidden, out.split(), out)

    def test_on_line_streams_merged_output_as_it_arrives(self):
        record = capsule.build(self.request())
        got, stamps = [], []
        import time as _t
        code = capsule.run(record, ["/bin/sh", "-c", "echo one; sleep 0.6; echo two >&2; echo three"], str(self.ws), [], {},
                           None, None, False, 20, "/workspace", None, on_line=lambda l: (got.append(l), stamps.append(_t.time())))
        self.assertEqual((code, got), (0, ["one", "two", "three"]))
        self.assertGreater(stamps[1] - stamps[0], 0.4)  # streamed, not delivered at exit

    def test_cli_round_trip(self):
        out = subprocess.run([sys.executable, str(Path(capsule.__file__)), "build", self.request()],
                             capture_output=True, text=True, env={**os.environ})
        self.assertEqual(out.returncode, 0, out.stderr)
        digest = json.loads(out.stdout)["capsule_digest"]
        run = subprocess.run([sys.executable, str(Path(capsule.__file__)), "run", digest[:12], "--workspace", str(self.ws),
                              "--", "hello"], capture_output=True, text=True, env={**os.environ})
        self.assertEqual(run.returncode, 0, run.stderr)
        self.assertIn("hello-from-layer", run.stdout)
        bad = subprocess.run([sys.executable, str(Path(capsule.__file__)), "run", "nomatch", "--", "true"],
                             capture_output=True, text=True, env={**os.environ})
        self.assertEqual(bad.returncode, 64)


if __name__ == "__main__":
    unittest.main()
