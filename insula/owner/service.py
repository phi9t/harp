"""Explicit Linux service owner for the supervisor's bounded Unix RPC contract.

Run as ``python -m insula.owner.service --config owner.local.json``. The operator
supplies an admitted capsule, readonly workspace and delegated cgroup root. No
host-process fallback, host environment inheritance, PID adoption or automatic
bootstrap replay exists. Only process probes and terminate/force stop are
currently supported. Namespace command/socket probes and secrets fail closed.
"""

from __future__ import annotations

import argparse
import fcntl
import json
import os
from pathlib import Path
import selectors
import signal
import socket
import sqlite3
import struct
import subprocess
import time
import uuid

from insula import capsule
from insula.cgroup import Cgroup
from .process import launch

MAX_FRAME = 1024 * 1024


class Refused(Exception):
    def __init__(self, code: str, message: str | None = None):
        self.error = {"code": code}
        if message is not None:
            self.error["message"] = message


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def absolute(value):
    path = Path(value)
    if not path.is_absolute() or ".." in path.parts or path.resolve() != path:
        raise ValueError(
            "owner paths must be canonical absolute paths without symlinks"
        )
    return path


def private(path):
    path = absolute(path)
    path.mkdir(mode=0o700, parents=True, exist_ok=True)
    stat = path.stat()
    if stat.st_uid != os.getuid() or stat.st_mode & 0o077:
        raise ValueError("owner directory must be private and owned by current uid")
    return path


def identity(pid, instance, generation):
    # We hold the unreaped direct child's Popen handle; numeric PID reuse cannot
    # occur during this read. Descendant cleanup relies on cgroup identity only.
    stat = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
    return {
        "owner_instance": instance,
        "boot_id": Path("/proc/sys/kernel/random/boot_id").read_text().strip(),
        "pid": pid,
        "start_time_ticks": int(stat[19]),
        "generation": generation,
    }


class Owner:
    def __init__(self, config):
        required = {"root_id", "authority_dir", "cgroup_root", "capsule", "workspace"}
        if set(config) != required:
            raise ValueError(
                "owner config requires exactly root_id, authority_dir, cgroup_root, capsule, workspace"
            )
        version = sqlite3.sqlite_version_info
        if not (
            version >= (3, 51, 3)
            or (version[:2] == (3, 50) and version >= (3, 50, 7))
            or (version[:2] == (3, 44) and version >= (3, 44, 6))
        ):
            raise RuntimeError("service owner requires SQLite with the WAL-reset fix")
        self.config = config
        self.root_id = config["root_id"]
        if (
            not isinstance(self.root_id, str)
            or not self.root_id
            or len(self.root_id) > 128
        ):
            raise ValueError("invalid root identity")
        self.authority = private(config["authority_dir"])
        self.workspace = absolute(config["workspace"])
        self.root_binding = Cgroup.validate_root(absolute(config["cgroup_root"]))
        # Anchor exclusivity to the actual kernel root, not a caller-selected
        # socket/state-directory alias. Durable activities use distinct subgroups.
        self.root_fd = os.open(
            config["cgroup_root"], os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
        )
        fcntl.flock(self.root_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        self._verify_capsule()
        dbpath = self.authority / "services.sqlite"
        if dbpath.is_symlink():
            raise ValueError("service database must not be symlinked")
        self.db = sqlite3.connect(dbpath, isolation_level=None)
        os.chmod(dbpath, 0o600)
        self.db.execute("PRAGMA journal_mode=WAL")
        self.db.execute("PRAGMA synchronous=FULL")
        self.db.executescript(
            """
          CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY,value TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS services(key TEXT PRIMARY KEY,generation TEXT NOT NULL,spec TEXT NOT NULL,
             cgroup TEXT,phase TEXT NOT NULL,observation TEXT);
          CREATE TABLE IF NOT EXISTS operations(id TEXT PRIMARY KEY,request TEXT NOT NULL,result TEXT);
        """
        )
        binding = canonical(
            {
                "root": self.root_binding,
                "root_id": self.root_id,
                "capsule": config["capsule"],
                "workspace": str(self.workspace),
            }
        )
        old = self.db.execute("SELECT value FROM meta WHERE key='binding'").fetchone()
        if old and old[0] != binding:
            raise ValueError("owner state belongs to another execution binding")
        self.db.execute("INSERT OR IGNORE INTO meta VALUES('binding',?)", (binding,))
        self.db.execute(
            "INSERT OR IGNORE INTO meta VALUES('instance',?)", (str(uuid.uuid4()),)
        )
        self.db.execute("INSERT OR IGNORE INTO meta VALUES('epoch','0')")
        self.instance = self.db.execute(
            "SELECT value FROM meta WHERE key='instance'"
        ).fetchone()[0]
        self.claim = None
        self.claim_socket = None
        self.children = {}
        self.selector = selectors.DefaultSelector()
        self.connections = {}
        self.logs = {}
        self.stopping = False
        # No orphan adoption or automatic service restart. Unknown launches are
        # reconciled to empty using retained cgroup identities before admission.
        self._cleanup_all()

    def _verify_capsule(self):
        record = self.config["capsule"]
        capsule.verify_host_layers(record)
        if (
            capsule.policy_user(record["policy"]) != "bench"
            or capsule.policy_identity(record["policy"])["users"]
        ):
            raise ValueError("service owner supports the fixed bench identity only")
        if not record.get("layers") or record["layers"][0]["kind"] != "rootfs":
            raise ValueError("service owner requires an admitted rootfs capsule")
        for layer in record["layers"]:
            if layer["kind"] == "tree":
                tree = capsule.store_root() / "layers" / layer["digest"] / "tree"
                if capsule.tree_digest(tree) != layer["digest"]:
                    raise ValueError("capsule tree changed")
            # No host-bound private control surfaces or credentials may leak.
            if layer["kind"] not in {"rootfs", "tree"}:
                raise ValueError(
                    "service capsule supports only sealed rootfs/tree layers"
                )

    def _group(self, row):
        return Cgroup.resume(
            json.loads(row[3]),
            attempt_id="service-" + uuid.UUID(row[1]).hex,
            root_identity=self.root_binding,
        )

    def _row(self, key):
        return self.db.execute(
            "SELECT key,generation,spec,cgroup,phase,observation FROM services WHERE key=?",
            (canonical(key),),
        ).fetchone()

    def _gone(self, row):
        result = {
            "state": "gone",
            "generation": row[1],
            "last_exit": None,
            "boundary_empty": True,
            "launch_resolved": True,
        }
        if row[3]:
            self._group(row).kill_and_wait()
        child = self.children.pop(row[0], None)
        if child is not None:
            child.wait(timeout=1)
            code = child.returncode
            result["last_exit"] = {
                "code": code if code >= 0 else None,
                "signal": -code if code < 0 else None,
                "observed_ms": int(time.time() * 1000),
            }
        self.db.execute(
            "UPDATE services SET phase='gone', observation=? WHERE key=?",
            (canonical(result), row[0]),
        )
        return result

    def _cleanup_all(self):
        for row in self.db.execute(
            "SELECT key,generation,spec,cgroup,phase,observation FROM services WHERE phase!='gone'"
        ).fetchall():
            self._gone(row)

    def validate(self, ownership):
        if self.claim is None or self.claim != ownership:
            raise Refused("ownership")
        # Recheck retained claim connection at effect time, not only after its
        # selector callback: a ready mutation cannot overtake a controller EOF.
        try:
            data = self.claim_socket.recv(1, socket.MSG_PEEK | socket.MSG_DONTWAIT)
            if not data:
                self._lose_claim()
                raise Refused("ownership")
            raise Refused("ownership")  # retained claim accepts no further frames
        except BlockingIOError:
            return

    def _lose_claim(self):
        self.claim = None
        old = self.claim_socket
        self.claim_socket = None
        self._cleanup_all()
        if old is not None:
            self._close(old)

    def observe(self, key, generation):
        row = self._row(key)
        if row is None:
            return {
                "state": "gone",
                "generation": None,
                "last_exit": None,
                "boundary_empty": True,
                "launch_resolved": True,
            }
        if generation is not None and generation != row[1]:
            raise Refused("stale_generation")
        if row[4] == "gone":
            return json.loads(row[5])
        child = self.children.get(row[0])
        if child is None:
            return {"state": "unknown"}
        if child.poll() is not None:
            return self._gone(row)
        return {
            "state": "alive",
            "identity": identity(child.pid, self.instance, row[1]),
        }

    def _reserve(self, request):
        op = str(uuid.UUID(request["operation_id"]))
        encoded = canonical(request)
        previous = self.db.execute(
            "SELECT request,result FROM operations WHERE id=?", (op,)
        ).fetchone()
        if previous is not None:
            if previous[0] != encoded:
                raise Refused("conflict", "operation id already names another request")
            if previous[1] is None:
                raise Refused("unknown")
            return op, json.loads(previous[1])
        self.db.execute("INSERT INTO operations VALUES(?,?,NULL)", (op, encoded))
        return op, None

    def _spec(self, spec):
        command = spec["command"]
        if spec["key"]["insula_id"] != self.root_id:
            raise Refused("unsupported", "only the admitted root is supported")
        if spec.get("cleanup_boundary", "service") != "service":
            raise Refused("unsupported", "requires cgroup service cleanup")
        if spec.get("controller_exit", "keep_services") != "stop_services":
            raise Refused(
                "unsupported", "controller-independent survival is not qualified"
            )
        if command.get("env_refs"):
            raise Refused("unsupported", "secret resolution is not implemented")
        if command["identity"] != {"uid": os.getuid(), "gid": os.getgid()}:
            raise Refused(
                "unsupported", "runtime identity must match admitted host bench uid/gid"
            )
        if (
            not 0 < spec["stop"]["deadline_ms"] <= 3000
            or type(spec["stop"]["force"]) is not bool
        ):
            raise Refused(
                "unsupported",
                "stop requires a bounded deadline <= 3000ms and explicit force policy",
            )
        if spec["stop"]["graceful"]["kind"] != "terminate":
            raise Refused("unsupported", "only terminate stop is supported")
        for role in ("readiness", "health"):
            if (
                spec.get(role, {}).get("probe", {"kind": "process"})["kind"]
                != "process"
            ):
                raise Refused(
                    "unsupported", "only bounded process probes are supported"
                )
        if (
            not 0 < spec["logs"]["max_bytes"] <= 64 * 1024 * 1024
            or spec["logs"]["retained_files"] != 2
        ):
            raise Refused(
                "unsupported", "exactly two bounded stream logs are supported"
            )
        for path in (command["executable"], command["cwd"]):
            if not str(path).startswith("/") or ".." in Path(path).parts:
                raise Refused(
                    "invalid", "command paths must be absolute without traversal"
                )
        if not (
            command["cwd"] == "/workspace" or command["cwd"].startswith("/workspace/")
        ):
            raise Refused("unsupported", "cwd must be under readonly workspace")
        if spec["adapter"]["kind"] not in {"foreground", "tmux"}:
            raise Refused("unsupported", "unknown adapter")
        if len(command["args"]) > 256 or any(
            not isinstance(a, str) or len(a) > 8192 or "\0" in a
            for a in command["args"]
        ):
            raise Refused("invalid", "invalid command arguments")

    def start(self, ownership, request):
        self._spec(request["spec"])
        self._verify_capsule()
        spec = request["spec"]
        generation = str(uuid.UUID(request["generation"]))
        key = canonical(spec["key"])
        row = self._row(spec["key"])
        if row is not None and row[4] != "gone":
            if row[1] == generation and row[2] == canonical(spec):
                return self.observe(spec["key"], generation)
            raise Refused("conflict", "service boundary still owned")
        if (
            self.db.execute(
                "SELECT count(*) FROM services WHERE phase!='gone'"
            ).fetchone()[0]
            >= 128
        ):
            raise Refused("unsupported", "maximum 128 active services")
        op, result = self._reserve({"launch": True, **request})
        if result is not None:
            return self.observe(spec["key"], generation)
        self.db.execute(
            "INSERT OR REPLACE INTO services VALUES(?,?,?,NULL,'reserved',NULL)",
            (key, generation, canonical(spec)),
        )
        group = Cgroup.create(
            self.config["cgroup_root"],
            "service-" + uuid.UUID(generation).hex,
            root_identity=self.root_binding,
        )
        self.db.execute(
            "UPDATE services SET cgroup=? WHERE key=?",
            (canonical(group.identity()), key),
        )
        directory = private(
            str(self.authority / ("service-" + uuid.UUID(generation).hex))
        )
        payload = private(str(directory / "payload"))
        command = spec["command"]
        argv = [command["executable"], *command["args"]]
        if spec["adapter"]["kind"] == "tmux":
            endpoint = spec["adapter"]["socket"]
            if (
                command["args"]
                or not endpoint.startswith("/home/bench/service/")
                or ".." in Path(endpoint).parts
            ):
                raise Refused(
                    "unsupported",
                    "tmux requires a fresh endpoint below service projection and no arguments",
                )
            # Each generation gets a distinct private host projection; all
            # endpoint names are reserved by this service's durable generation.
            argv = [
                command["executable"],
                "-D",
                "-f",
                "/dev/null",
                "-S",
                endpoint,
            ]
        namespace = capsule.run_argv(
            self.config["capsule"],
            str(self.workspace),
            [(str(payload), "/home/bench/service")],
            {
                "PATH": "/usr/bin:/bin",
                "HOME": "/home/bench",
                "LANG": "C.UTF-8",
                "LC_ALL": "C.UTF-8",
            },
            True,
            net="none",
            cwd=command["cwd"],
        )
        namespace += ["--tmpfs", "/sys/fs/cgroup", "--", *argv]
        self.validate(ownership)

        def joined():
            # Keep the root flock inherited through fork until cgroup membership
            # is proven. Recovery cannot overtake an in-flight preexec child.
            os.close(self.root_fd)

        child = launch(
            namespace,
            group=group,
            after_join=joined,
            cwd=self.workspace,
            env={"PATH": "/usr/bin:/bin"},
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            pass_fds=(self.root_fd,),
            start_new_session=True,
        )
        self.children[key] = child
        if row is not None and row[1] != generation:
            # LogPolicy retains exactly the two current stream files. Payloads
            # remain for deliberate workload checkpoint/recovery decisions.
            previous = self.authority / ("service-" + uuid.UUID(row[1]).hex)
            for name in ("stdout.log", "stderr.log"):
                (previous / name).unlink(missing_ok=True)
        for name, stream in (("stdout", child.stdout), ("stderr", child.stderr)):
            os.set_blocking(stream.fileno(), False)
            sink = open(directory / (name + ".log"), "xb", buffering=0)
            os.chmod(directory / (name + ".log"), 0o600)
            self.logs[stream.fileno()] = [stream, sink, spec["logs"]["max_bytes"], 0]
            self.selector.register(stream, selectors.EVENT_READ, ("log", None))
        self.db.execute("UPDATE services SET phase='running' WHERE key=?", (key,))
        result = self.observe(spec["key"], generation)
        self.db.execute(
            "UPDATE operations SET result=? WHERE id=?", (canonical(result), op)
        )
        return result

    def stop(self, ownership, request):
        row = self._row(request["key"])
        if row is None:
            raise Refused("not_found")
        if row[1] != request["expected_generation"]:
            raise Refused("stale_generation")
        policy = request["policy"]
        if (
            policy["graceful"]["kind"] != "terminate"
            or not 0 < policy["deadline_ms"] <= 3000
        ):
            raise Refused(
                "unsupported", "stop requires terminate and deadline <= 3000ms"
            )
        op, result = self._reserve({"stop": True, **request})
        if result is not None:
            return result
        self.validate(ownership)
        child = self.children.get(row[0])
        if child is not None and child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=policy["deadline_ms"] / 1000)
            except subprocess.TimeoutExpired:
                if not policy["force"]:
                    raise Refused("unknown")
        if not policy["force"] and self._group(row).observation()["populated"]:
            raise Refused("unknown")
        result = self._gone(row)
        self.db.execute(
            "UPDATE operations SET result=? WHERE id=?", (canonical(result), op)
        )
        return result

    def dispatch(self, conn, request):
        method = request["method"]
        if method in {"capabilities", "claim"}:
            if request["root_insula_id"] != self.root_id:
                raise Refused("unsupported", "unknown root")
        if method == "capabilities":
            return "capabilities", {
                "owner_instance": self.instance,
                "root_insula_id": self.root_id,
                "canonical_authority_dir": str(self.authority),
                "generation_fencing": True,
                "controller_fencing": True,
                "durable_launch_reservations": True,
                "service_boundary_cleanup": True,
                "bounded_probes": True,
                "bounded_logs": True,
                "survives_controller_exit": False,
                "details": [
                    "cgroup-v2 service boundary; process probes only; no secrets; stop_services required"
                ],
            }
        if method == "claim":
            if self.claim is not None:
                self.validate(self.claim)
                raise Refused("ownership")
            epoch = (
                int(
                    self.db.execute(
                        "SELECT value FROM meta WHERE key='epoch'"
                    ).fetchone()[0]
                )
                + 1
            )
            self.db.execute("UPDATE meta SET value=? WHERE key='epoch'", (str(epoch),))
            self.claim = {
                "root_insula_id": self.root_id,
                "owner_instance": self.instance,
                "controller_id": str(uuid.UUID(request["controller_id"])),
                "epoch": epoch,
            }
            self.claim_socket = conn
            return "claimed", self.claim
        self.validate(request["ownership"])
        if method == "validate":
            return "acknowledged", None
        if method == "release":
            self._lose_claim()
            return "acknowledged", None
        if method == "observe":
            return "observation", self.observe(
                request["key"], request.get("generation")
            )
        if method == "launch":
            return "observation", self.start(request["ownership"], request["request"])
        if method == "stop":
            return "observation", self.stop(request["ownership"], request["request"])
        if method == "probe":
            probe = request["request"]
            if probe["spec"]["probe"]["kind"] != "process":
                raise Refused("unsupported", "only process probes are supported")
            alive = self.observe(probe["key"], probe["generation"])["state"] == "alive"
            return "probe", {
                "generation": probe["generation"],
                "class": "success" if alive else "target_unavailable",
                "code": "process_alive" if alive else "target_unavailable",
                "panes": [],
                "expectations_met": None,
                "observed_ms": int(time.time() * 1000),
            }
        raise Refused("invalid", "unknown method")

    def _close(self, conn):
        self.connections.pop(conn, None)
        try:
            self.selector.unregister(conn)
        except KeyError:
            pass
        conn.close()

    def serve(self):
        socket_path = self.authority / "owner.sock"
        if socket_path.exists() or socket_path.is_symlink():
            # Root flock held: no alternate live owner can own this endpoint.
            if not socket_path.is_socket():
                raise ValueError("owner endpoint is not a socket")
            socket_path.unlink()
        listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        listener.bind(str(socket_path))
        os.chmod(socket_path, 0o600)
        listener.listen(32)
        listener.setblocking(False)
        self.selector.register(listener, selectors.EVENT_READ, ("accept", None))
        try:
            while not self.stopping:
                for event, _mask in self.selector.select(0.05):
                    kind, _ = event.data
                    conn = event.fileobj
                    if kind == "accept":
                        client, _ = listener.accept()
                        client.setblocking(False)
                        peer = struct.unpack(
                            "3i",
                            client.getsockopt(
                                socket.SOL_SOCKET, socket.SO_PEERCRED, 12
                            ),
                        )
                        if peer[1] != os.getuid() or len(self.connections) >= 64:
                            client.close()
                            continue
                        self.connections[client] = [bytearray(), time.monotonic()]
                        self.selector.register(
                            client, selectors.EVENT_READ, ("rpc", None)
                        )
                    elif kind == "log":
                        entry = self.logs[conn.fileno()]
                        chunk = conn.read(65536)
                        if chunk:
                            allowed = max(0, entry[2] - entry[3])
                            entry[1].write(chunk[:allowed])
                            entry[3] += min(allowed, len(chunk))
                        elif chunk == b"":
                            self.selector.unregister(conn)
                            self.logs.pop(conn.fileno())
                            entry[1].close()
                            conn.close()
                    elif conn is self.claim_socket:
                        self._lose_claim()
                    else:
                        self._rpc(conn)
                for conn, (_buffer, opened) in list(self.connections.items()):
                    if conn is not self.claim_socket and time.monotonic() - opened > 3:
                        self._close(conn)
        finally:
            self._lose_claim()
            self._cleanup_all()
            for stream, sink, _limit, _used in self.logs.values():
                stream.close()
                sink.close()
            for conn in list(self.connections):
                self._close(conn)
            listener.close()
            socket_path.unlink(missing_ok=True)
            self.db.close()
            os.close(self.root_fd)

    def _rpc(self, conn):
        try:
            chunk = conn.recv(65536)
            if not chunk:
                self._close(conn)
                return
            buffer = self.connections[conn][0]
            buffer.extend(chunk)
            if len(buffer) < 4:
                return
            size = struct.unpack("!I", buffer[:4])[0]
            if not 0 < size <= MAX_FRAME or len(buffer) > size + 4:
                self._close(conn)
                return
            if len(buffer) < size + 4:
                return
            envelope = json.loads(buffer[4:])
            if (
                set(envelope) != {"version", "request_id", "request"}
                or envelope["version"] != 1
            ):
                self._close(conn)
                return
            request_id = str(uuid.UUID(envelope["request_id"]))
            try:
                kind, value = self.dispatch(conn, envelope["request"])
                reply = {"kind": kind}
                if value is not None:
                    reply["value"] = value
                response = {"status": "success", "value": reply}
            except Refused as error:
                response = {"status": "error", "error": error.error}
            except (ValueError, KeyError, TypeError):
                response = {
                    "status": "error",
                    "error": {"code": "invalid", "message": "invalid owner request"},
                }
            except (OSError, RuntimeError, sqlite3.Error):
                response = {"status": "error", "error": {"code": "unknown"}}
            payload = canonical(
                {"version": 1, "request_id": request_id, "response": response}
            ).encode()
            conn.settimeout(0.1)
            conn.sendall(struct.pack("!I", len(payload)) + payload)
            conn.setblocking(False)
            if conn is not self.claim_socket:
                self._close(conn)
        except (OSError, ValueError, TypeError, KeyError):
            if conn is self.claim_socket:
                self._lose_claim()
            else:
                self._close(conn)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", required=True)
    args = parser.parse_args()
    owner = Owner(json.loads(Path(args.config).read_text()))

    def stop(_sig, _frame):
        owner.stopping = True

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    owner.serve()


if __name__ == "__main__":
    main()
