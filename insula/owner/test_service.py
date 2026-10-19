"""Real Linux owner protocol tests; explicit delegated cgroup/rootfs required.

Run with INSULA_OWNER_NATIVE=1, HARP_TEST_CGROUP_ROOT, HARP_TEST_ROOTFS and
INSULA_SUPERVISOR_BIN. Native mode fails missing prerequisites rather than skips.
Ordinary discovery skips these Linux kernel tests on portable hosts.
"""

import copy
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import sys
import tempfile
import time
import unittest
import uuid

from insula import capsule


class Rpc:
    def __init__(self, path):
        self.path = str(path)
        self.claim_socket = None
        self.ownership = None

    def call(self, method, retain=False, **kwargs):
        stream = socket.socket(socket.AF_UNIX)
        stream.settimeout(5)
        stream.connect(self.path)
        request = {
            "version": 1,
            "request_id": str(uuid.uuid4()),
            "request": {"method": method, **kwargs},
        }
        data = json.dumps(request).encode()
        stream.sendall(struct.pack("!I", len(data)) + data)

        def exact(n):
            data = b""
            while len(data) < n:
                more = stream.recv(n - len(data))
                if not more:
                    raise RuntimeError("owner closed incomplete frame")
                data += more
            return data

        size = struct.unpack("!I", exact(4))[0]
        response = json.loads(exact(size))
        assert response["request_id"] == request["request_id"]
        if retain and response["response"]["status"] == "success":
            self.claim_socket = stream
            self.ownership = response["response"]["value"]["value"]
        else:
            stream.close()
        return response["response"]

    def owned(self, method, **kwargs):
        return self.call(method, ownership=self.ownership, **kwargs)

    def close(self):
        if self.claim_socket:
            self.claim_socket.close()
            self.claim_socket = None


@unittest.skipUnless(
    os.environ.get("INSULA_OWNER_NATIVE") == "1",
    "requires explicit native owner qualification",
)
class NativeOwner(unittest.TestCase):
    def setUp(self):
        for name in (
            "HARP_TEST_CGROUP_ROOT",
            "HARP_TEST_ROOTFS",
            "INSULA_SUPERVISOR_BIN",
        ):
            self.assertTrue(
                os.environ.get(name), f"native qualification requires {name}"
            )
        self.directory = tempfile.TemporaryDirectory(prefix="insula-owner-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.workspace = self.root / "workspace"
        self.workspace.mkdir()
        self.authority = self.root / "authority"
        self.authority.mkdir(mode=0o700)
        self.capsule_store = self.root / "capsules"
        self.old_store = os.environ.get("INSULA_CAPSULES")
        os.environ["INSULA_CAPSULES"] = str(self.capsule_store)
        self.addCleanup(self.restore_store)
        request = self.root / "capsule.json"
        request.write_text(
            json.dumps(
                {
                    "layers": [
                        {
                            "kind": "rootfs",
                            "name": "native",
                            "source": os.environ["HARP_TEST_ROOTFS"],
                        }
                    ],
                    "policy": {"net": "none", "gpu": False},
                }
            )
        )
        record = capsule.build(str(request))
        self.config = {
            "root_id": "root",
            "authority_dir": str(self.authority),
            "cgroup_root": os.environ["HARP_TEST_CGROUP_ROOT"],
            "capsule": record,
            "workspace": str(self.workspace),
        }
        self.config_path = self.root / "owner.json"
        self.config_path.write_text(json.dumps(self.config))
        self.processes = []
        self.clients = []
        self.addCleanup(self.cleanup)
        self.owner = self.start_owner()
        self.rpc = Rpc(self.authority / "owner.sock")
        self.clients.append(self.rpc)
        self.assertEqual(
            self.rpc.call(
                "claim",
                retain=True,
                root_insula_id="root",
                controller_id=str(uuid.uuid4()),
            )["status"],
            "success",
        )

    def restore_store(self):
        if self.old_store is None:
            os.environ.pop("INSULA_CAPSULES", None)
        else:
            os.environ["INSULA_CAPSULES"] = self.old_store

    def cleanup(self):
        for rpc in self.clients:
            rpc.close()
        for proc in reversed(self.processes):
            if proc.poll() is None:
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait()
            if proc.stderr is not None:
                proc.stderr.close()

    def start_owner(self):
        log = open(self.root / ("owner-" + str(len(self.processes)) + ".log"), "wb")
        self.addCleanup(log.close)
        proc = subprocess.Popen(
            [
                sys.executable,
                "-m",
                "insula.owner.service",
                "--config",
                str(self.config_path),
            ],
            stdout=log,
            stderr=log,
        )
        self.processes.append(proc)
        deadline = time.monotonic() + 4
        while time.monotonic() < deadline:
            if proc.poll() is not None:
                self.fail(
                    (
                        self.root / ("owner-" + str(len(self.processes) - 1) + ".log")
                    ).read_text()
                )
            if (self.authority / "owner.sock").is_socket():
                try:
                    with socket.socket(socket.AF_UNIX) as sock:
                        sock.connect(str(self.authority / "owner.sock"))
                    return proc
                except OSError:
                    pass
            time.sleep(0.02)
        self.fail("owner socket never ready")

    def request(self, code="import time; time.sleep(60)", name="service"):
        generation = str(uuid.uuid4())
        return {
            "operation_id": str(uuid.uuid4()),
            "generation": generation,
            "spec": {
                "key": {"insula_id": "root", "service_id": name},
                "revision": "test-1",
                "command": {
                    "executable": "/usr/bin/python3",
                    "args": ["-c", code],
                    "cwd": "/workspace",
                    "env_refs": {},
                    "identity": {"uid": os.getuid(), "gid": os.getgid()},
                },
                "adapter": {"kind": "foreground"},
                "readiness": {
                    "probe": {"kind": "process"},
                    "interval_ms": 100,
                    "timeout_ms": 200,
                    "failure_threshold": 1,
                    "max_output_bytes": 1024,
                },
                "health": {
                    "probe": {"kind": "process"},
                    "interval_ms": 100,
                    "timeout_ms": 200,
                    "failure_threshold": 1,
                    "max_output_bytes": 1024,
                },
                "startup_grace_ms": 0,
                "restart": {"kind": "never"},
                "replay_safe": False,
                "stop": {
                    "graceful": {"kind": "terminate"},
                    "deadline_ms": 500,
                    "force": True,
                },
                "logs": {"max_bytes": 2048, "retained_files": 2},
                "cleanup_boundary": "service",
                "controller_exit": "stop_services",
            },
        }

    def launch(self, request):
        response = self.rpc.owned("launch", request=request)
        self.assertEqual(response["status"], "success", response)
        self.assertEqual(response["value"]["value"]["state"], "alive", response)
        return response["value"]["value"]

    def stop(self, request):
        return self.rpc.owned(
            "stop",
            request={
                "operation_id": str(uuid.uuid4()),
                "key": request["spec"]["key"],
                "expected_generation": request["generation"],
                "policy": request["spec"]["stop"],
            },
        )

    def wait_file(self, path):
        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            if path.exists():
                return path.read_text()
            time.sleep(0.02)
        self.fail(f"payload output missing: {path}")

    def test_real_namespace_descendants_and_stale_generation(self):
        req = self.request(
            "import os,pathlib,time; p=pathlib.Path('/home/bench/service/proof'); p.write_text(str(os.readlink('/proc/self/ns/pid'))+'\\n'+str(pathlib.Path('/sys/fs/cgroup/cgroup.procs').exists())); pid=os.fork(); os.setsid() if pid==0 else None; time.sleep(60)"
        )
        first = self.launch(req)
        duplicate = self.launch(req)
        self.assertEqual(first["identity"], duplicate["identity"])
        proof = self.wait_file(
            self.authority
            / ("service-" + uuid.UUID(req["generation"]).hex)
            / "payload/proof"
        )
        self.assertNotEqual(proof.splitlines()[0], os.readlink("/proc/self/ns/pid"))
        self.assertEqual(proof.splitlines()[1], "False")
        stale = copy.deepcopy(req)
        stale["generation"] = str(uuid.uuid4())
        self.assertEqual(self.stop(stale)["error"]["code"], "stale_generation")
        stopped = self.stop(req)
        self.assertTrue(stopped["value"]["value"]["boundary_empty"])
        self.assertTrue(stopped["value"]["value"]["launch_resolved"])
        newer = self.request()
        self.launch(newer)
        self.assertEqual(self.stop(req)["error"]["code"], "stale_generation")
        self.stop(newer)

    def test_claim_loss_fences_effect_and_cleans_boundary(self):
        req = self.request()
        self.launch(req)
        old = copy.deepcopy(self.rpc.ownership)
        self.rpc.close()
        time.sleep(0.1)
        other = Rpc(self.authority / "owner.sock")
        self.clients.append(other)
        claimed = other.call(
            "claim", retain=True, root_insula_id="root", controller_id=str(uuid.uuid4())
        )
        self.assertEqual(claimed["status"], "success", claimed)
        self.assertGreater(other.ownership["epoch"], old["epoch"])
        self.assertEqual(
            other.call("launch", ownership=old, request=self.request())["error"][
                "code"
            ],
            "ownership",
        )
        observed = other.owned(
            "observe", key=req["spec"]["key"], generation=req["generation"]
        )
        self.assertTrue(observed["value"]["value"]["boundary_empty"])

    def test_lost_launch_reply_reconciles_without_duplicate_effect(self):
        request = self.request()
        envelope = {
            "version": 1,
            "request_id": str(uuid.uuid4()),
            "request": {
                "method": "launch",
                "ownership": self.rpc.ownership,
                "request": request,
            },
        }
        data = json.dumps(envelope).encode()
        stream = socket.socket(socket.AF_UNIX)
        stream.connect(self.rpc.path)
        stream.sendall(struct.pack("!I", len(data)) + data)
        stream.shutdown(socket.SHUT_WR)
        stream.close()
        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            response = self.rpc.owned(
                "observe", key=request["spec"]["key"], generation=request["generation"]
            )
            observed = response["value"]["value"]
            if observed["state"] == "alive":
                break
            time.sleep(0.02)
        self.assertEqual(observed["state"], "alive")
        replayed = self.launch(request)
        self.assertEqual(observed["identity"], replayed["identity"])
        self.stop(request)

    def test_owner_crash_reconciles_cgroup_without_replaying_bootstrap(self):
        req = self.request(
            "import pathlib,time; p=pathlib.Path('/home/bench/service/count'); p.write_text('1'); time.sleep(60)"
        )
        self.launch(req)
        counter = (
            self.authority
            / ("service-" + uuid.UUID(req["generation"]).hex)
            / "payload/count"
        )
        self.assertEqual(self.wait_file(counter), "1")
        self.owner.kill()
        self.owner.wait()
        self.rpc.close()
        self.owner = self.start_owner()
        new = Rpc(self.authority / "owner.sock")
        self.clients.append(new)
        self.assertEqual(
            new.call(
                "claim",
                retain=True,
                root_insula_id="root",
                controller_id=str(uuid.uuid4()),
            )["status"],
            "success",
        )
        observation = new.owned(
            "observe", key=req["spec"]["key"], generation=req["generation"]
        )
        self.assertTrue(observation["value"]["value"]["boundary_empty"])
        self.assertEqual(counter.read_text(), "1")

    def test_root_alias_cannot_create_second_owner(self):
        alias = dict(self.config)
        alias["authority_dir"] = str(self.root / "other-state")
        config = self.root / "alias.json"
        config.write_text(json.dumps(alias))
        result = subprocess.run(
            [sys.executable, "-m", "insula.owner.service", "--config", str(config)],
            capture_output=True,
            timeout=5,
        )
        self.assertNotEqual(result.returncode, 0)
        caps = self.rpc.call("capabilities", root_insula_id="root")
        self.assertFalse(caps["value"]["value"]["survives_controller_exit"])

    def test_bounded_logs_and_unsupported_capabilities(self):
        req = self.request(
            "import os,time; os.write(1,b'x'*200000); os.write(2,b'y'*200000); time.sleep(60)"
        )
        self.launch(req)
        time.sleep(0.2)
        directory = self.authority / ("service-" + uuid.UUID(req["generation"]).hex)
        self.assertEqual((directory / "stdout.log").stat().st_size, 2048)
        self.assertEqual((directory / "stderr.log").stat().st_size, 2048)
        self.stop(req)
        for mutate in (
            lambda s: s.update(controller_exit="keep_services"),
            lambda s: s["command"].update(env_refs={"KEY": "secret"}),
            lambda s: s.update(cleanup_boundary="direct_process"),
        ):
            bad = self.request(name=str(uuid.uuid4()))
            mutate(bad["spec"])
            self.assertEqual(
                self.rpc.owned("launch", request=bad)["error"]["code"], "unsupported"
            )

    def test_tmux_reconnect_and_empty_replacement_do_not_replay_pane(self):
        request = self.request()
        request["spec"]["command"].update(executable="/usr/bin/tmux", args=[])
        request["spec"]["adapter"] = {
            "kind": "tmux",
            "socket": "/home/bench/service/tmux.sock",
        }
        self.launch(request)
        payload = (
            self.authority
            / ("service-" + uuid.UUID(request["generation"]).hex)
            / "payload"
        )
        endpoint = payload / "tmux.sock"
        deadline = time.monotonic() + 3
        while not endpoint.exists() and time.monotonic() < deadline:
            time.sleep(0.02)
        self.assertTrue(
            endpoint.is_socket(), (payload.parent / "stderr.log").read_text()
        )

        # The managed host temporary path can exceed sockaddr_un's limit.
        # Resolve through a held directory FD; the server's namespace endpoint
        # stays short and this test does not install a host socket alias.
        payload_fd = os.open(payload, os.O_RDONLY | os.O_DIRECTORY)
        self.addCleanup(os.close, payload_fd)
        client_endpoint = f"/proc/self/fd/{payload_fd}/tmux.sock"

        def client(*args):
            result = subprocess.run(
                [
                    os.environ.get("INSULA_TEST_TMUX", "/usr/bin/tmux"),
                    "-N",
                    "-f",
                    "/dev/null",
                    "-S",
                    client_endpoint,
                    *args,
                ],
                capture_output=True,
                text=True,
                timeout=3,
                pass_fds=(payload_fd,),
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            return result.stdout

        client(
            "new-session",
            "-d",
            "-s",
            "once",
            "/bin/sh",
            "-c",
            "echo 1 > /home/bench/service/count; sleep 60",
        )
        counter = payload / "count"
        self.assertEqual(self.wait_file(counter).strip(), "1")
        before = client("list-panes", "-a", "-F", "#{pane_id}:#{pane_pid}")
        self.assertEqual(
            client("list-panes", "-a", "-F", "#{pane_id}:#{pane_pid}"), before
        )
        self.assertEqual(counter.read_text().strip(), "1")
        self.stop(request)
        replacement = self.request()
        replacement["spec"]["command"].update(executable="/usr/bin/tmux", args=[])
        replacement["spec"]["adapter"] = request["spec"]["adapter"]
        self.launch(replacement)
        self.assertEqual(counter.read_text().strip(), "1")
        newer = (
            self.authority
            / ("service-" + uuid.UUID(replacement["generation"]).hex)
            / "payload"
        )
        self.assertFalse((newer / "count").exists())
        self.stop(replacement)

    def test_rust_supervisor_uses_real_execution_owner(self):
        self.rpc.close()
        time.sleep(0.1)
        binary = os.environ["INSULA_SUPERVISOR_BIN"]
        state = self.root / "supervisor"
        state.mkdir(mode=0o700)
        proc = subprocess.Popen(
            [
                binary,
                "serve",
                "--owner-socket",
                str(self.authority / "owner.sock"),
                "--root-id",
                "root",
                "--state-dir",
                str(state),
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        self.processes.append(proc)
        socket_path = state / "control.sock"
        deadline = time.monotonic() + 3
        while not socket_path.is_socket() and time.monotonic() < deadline:
            if proc.poll() is not None:
                self.fail(proc.stderr.read().decode())
            time.sleep(0.02)
        spec = self.request()["spec"]
        path = self.root / "service.json"
        path.write_text(json.dumps(spec))
        result = subprocess.run(
            [
                binary,
                "start",
                "--socket",
                str(socket_path),
                "--spec",
                str(path),
                "--operation-id",
                str(uuid.uuid4()),
            ],
            capture_output=True,
            text=True,
            timeout=6,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("generation", result.stdout)
        proc.terminate()
        proc.wait(timeout=5)


if __name__ == "__main__":
    unittest.main()
