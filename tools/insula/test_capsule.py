"""capsule.py against real bwrap in a throwaway store (no network, no GPU needed)."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import capsule  # noqa: E402


class CapsuleTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        os.environ["CORENIUS_CAPSULES"] = str(self.root / "store")
        self.tool = self.root / "tool"
        (self.tool / "bin").mkdir(parents=True)
        (self.tool / "bin" / "hello").write_text("#!/bin/sh\necho hello-from-layer\n")
        (self.tool / "bin" / "hello").chmod(0o755)
        self.hostdir = self.root / "hostro"
        self.hostdir.mkdir()
        (self.hostdir / "f").write_text("host\n")
        self.ws = self.root / "ws"
        self.ws.mkdir()

    def tearDown(self):
        os.environ.pop("CORENIUS_CAPSULES", None)
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
            argv = capsule.run_argv(record, str(self.ws), kw.get("rw", []), kw.get("env", {}), False, kw.get("net"))
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
        self.assertIn("CORENIUS_IN_INSULA=1", out)
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
        request = Path(capsule.__file__).resolve().parents[2] / "bench/insula/requests/agent.json"
        if not request.exists():
            self.skipTest("no agent request")
        record = capsule.build(str(request))
        code, out = self.sh(record, "ls -A ~/.cargo 2>&1; ls -A /home/bench/.cargo 2>&1; ls -A " +
                            str(Path.home() / ".cargo") + " 2>&1")
        for hidden in (".codex", ".trae", ".agents", ".git", "registry", "credentials.toml"):
            self.assertNotIn(hidden, out.split(), out)

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
