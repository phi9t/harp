#!/usr/bin/env python3
"""Crash Pi after a real controller commit, withholding its reply. No models."""

import argparse
import hashlib
import json
import os
import pathlib
import secrets
import shutil
import signal
import socket
import sqlite3
import subprocess
import tempfile
import threading
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
PIN = "ae92585d3b3e5f1e4b123d14a34314d826d8d9f5"
CONTROLLER_PIN = "d7f674829e9844d011869b711d64fcf55313f7f1"


def digest(value):
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def rpc(path, request):
    with socket.socket(socket.AF_UNIX) as s:
        s.settimeout(10)
        s.connect(str(path))
        s.sendall((json.dumps(request) + "\n").encode())
        return json.loads(s.makefile("rb").readline())


def wait(predicate, seconds=30):
    end = time.monotonic() + seconds
    while not predicate():
        if time.monotonic() > end:
            raise TimeoutError("probe barrier timed out")
        time.sleep(0.02)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--pi-environment", type=pathlib.Path, required=True)
    p.add_argument("--upstream", type=pathlib.Path, required=True)
    p.add_argument(
        "--output",
        type=pathlib.Path,
        default=ROOT
        / "docs/efforts/pi-env-durable/evidence/interrupted-effect-probe/result.json",
    )
    a = p.parse_args()
    scratch = ROOT / ".scratch/pi-effect-probe"
    scratch.mkdir(parents=True, exist_ok=True)
    actual_pin = subprocess.check_output(
        ["git", "-C", str(a.upstream), "rev-parse", "HEAD"], text=True
    ).strip()
    assert actual_pin == PIN, ("wrong upstream commit", actual_pin)
    controller_hashes = {}
    controller_files = subprocess.check_output(
        [
            "git",
            "-C",
            str(ROOT),
            "ls-tree",
            "-r",
            "--name-only",
            CONTROLLER_PIN,
            "controller/src",
            "controller/Cargo.toml",
            "Cargo.lock",
        ],
        text=True,
    ).splitlines()
    for relative in controller_files:
        original = subprocess.check_output(
            ["git", "-C", str(ROOT), "show", f"{CONTROLLER_PIN}:{relative}"]
        )
        assert (ROOT / relative).read_bytes() == original, (
            "modified controller input",
            relative,
        )
        controller_hashes[relative] = hashlib.sha256(original).hexdigest()
    envdir = pathlib.Path(tempfile.mkdtemp(prefix="environment-", dir=scratch))
    # Only dependency bytes come from the prepared offline cache. Runtime sources
    # are copied from the explicitly pinned upstream checkout, never modified.
    shutil.copytree(
        a.pi_environment / "node_modules", envdir / "node_modules", symlinks=True
    )
    for name in ["package.json", "package-lock.json"]:
        shutil.copy2(a.pi_environment / name, envdir / name)
    source_hashes = {}
    for package in ["chord", "ai", "durable", "telemetry"]:
        src = a.upstream / "packages" / package
        dest = envdir / "packages" / package
        shutil.copytree(src, dest, symlinks=True)
        for f in [dest / "package.json", *sorted((dest / "src").rglob("*"))]:
            if f.is_file():
                relative = str(f.relative_to(envdir))
                original = subprocess.check_output(
                    ["git", "-C", str(a.upstream), "show", f"{PIN}:{relative}"]
                )
                assert f.read_bytes() == original, ("modified runtime source", relative)
                source_hashes[relative] = hashlib.sha256(original).hexdigest()
        # These two packages lack source conditions in published manifests.
        # Add test-time resolution only; no TypeScript source edits.
        if package in ["ai", "telemetry"]:
            manifest = dest / "package.json"
            value = json.loads(manifest.read_text())
            for key, export in value["exports"].items():
                if isinstance(export, dict) and export.get("import", "").startswith(
                    "./dist/"
                ):
                    export = {
                        "source": export["import"]
                        .replace("./dist/", "./src/")
                        .replace(".js", ".ts"),
                        **export,
                    }
                    value["exports"][key] = export
            manifest.write_text(json.dumps(value, indent=2) + "\n")
    package_links = {
        "chord": "chord",
        "pi-ai": "ai",
        "pi-durable": "durable",
        "pi-telemetry": "telemetry",
    }
    namespace = envdir / "node_modules/@earendil-works"
    namespace.mkdir(parents=True, exist_ok=True)
    for name, package in package_links.items():
        link = namespace / name
        if link.is_symlink() or link.is_file():
            link.unlink()
        elif link.is_dir():
            shutil.rmtree(link)
        link.symlink_to(envdir / "packages" / package, target_is_directory=True)
        assert link.resolve() == envdir / "packages" / package
    shutil.copy2(
        ROOT / "experiments/pi-env-durable/effect-worker.ts",
        envdir / "effect-worker.ts",
    )
    binary = scratch / "target/debug/corenius-sqlite-controller"
    subprocess.run(
        [
            "cargo",
            "build",
            "--locked",
            "--offline",
            "--manifest-path",
            str(ROOT / "controller/Cargo.toml"),
            "--target-dir",
            str(scratch / "target"),
            "--bin",
            "corenius-sqlite-controller",
        ],
        check=True,
    )
    results = []
    processes = []
    case_roots = []
    sockets = []
    threads = []
    releases = []
    fds = []
    logs = []
    try:
        for revoked in [False, True]:
            print(
                "Running "
                + ("revoked authority" if revoked else "lost acknowledgement")
                + " scenario",
                flush=True,
            )
            # Short path avoids the Unix socket length limit. All state stays in owned scratch.
            case = pathlib.Path(tempfile.mkdtemp(prefix="run-", dir=scratch))
            os.chmod(case, 0o700)
            case_roots.append(case)
            # Bind socket through /proc/self/fd using an opened private directory, keeping address short.
            fd = os.open(case, os.O_RDONLY)
            fds.append(fd)
            sock = f"/proc/{os.getpid()}/fd/{fd}/c.sock"
            proxy = f"/proc/{os.getpid()}/fd/{fd}/p.sock"
            token = secrets.token_hex(32)
            sterile = {
                "PATH": os.environ["PATH"],
                "HOME": str(case),
                "TMPDIR": str(case),
                "LANG": "C.UTF-8",
            }
            log = open(case / "controller.log", "w")
            logs.append(log)
            daemon = subprocess.Popen(
                [str(binary), str(case / "controller.sqlite"), sock],
                env={**sterile, "CORENIUS_OPERATOR_TOKEN": token},
                stdout=log,
                stderr=log,
            )
            processes.append(daemon)
            wait(lambda: pathlib.Path(sock).exists() or daemon.poll() is not None)
            assert daemon.poll() is None
            auth = {"principal": "operator", "token": token}
            revision = 0

            def command(kind, args, key=None, principal=None):
                nonlocal revision
                r = rpc(
                    sock,
                    {
                        "auth": principal or auth,
                        "project": "probe",
                        "command": {
                            "kind": kind,
                            "expected_revision": revision,
                            "idempotency_key": key or secrets.token_hex(8),
                            "args": args,
                        },
                    },
                )
                assert "ok" in r, r
                revision = r["ok"]["revision"]
                return r["ok"]

            command(
                "create_project",
                {"name": "probe", "base": "0123456789abcdef0123456789abcdef01234567"},
            )
            command(
                "create_task",
                {
                    "task": "work",
                    "work_ref": {
                        "deployment": "fixture",
                        "project": "probe",
                        "item": "work",
                    },
                    "source_revision": "fixture-v1",
                    "brief": "Model-free probe",
                },
            )
            grant = command("admit", {"task": "work"})["result"]
            worker_auth = {"principal": grant["principal"], "token": grant["token"]}
            captured = []
            ready = threading.Event()
            release = threading.Event()
            releases.append(release)
            errors = []
            listener = socket.socket(socket.AF_UNIX)
            sockets.append(listener)
            listener.settimeout(10)
            listener.bind(proxy)
            listener.listen()

            def intercept():
                try:
                    with listener.accept()[0] as client:
                        request = json.loads(client.makefile("rb").readline())
                        reply = rpc(sock, request)
                        assert "ok" in reply, reply
                        captured.append((request, reply))
                        ready.set()
                        release.wait(30)
                        # Deliberately never deliver the reply, even after the worker is killed.
                except BaseException as e:
                    errors.append(str(e))
                    ready.set()

            thread = threading.Thread(target=intercept)
            threads.append(thread)
            thread.start()
            worker_env = {
                **sterile,
                "PROBE_ROOT": str(case),
                "PROBE_SOCKET": proxy,
                "PROBE_PRINCIPAL": worker_auth["principal"],
                "PROBE_TOKEN": worker_auth["token"],
                "PROBE_REVISION": str(revision),
            }

            def launch(mode, env):
                out = open(case / (mode + ".log"), "w")
                logs.append(out)
                child = subprocess.Popen(
                    [
                        "node",
                        "--conditions=source",
                        "--experimental-strip-types",
                        str(envdir / "effect-worker.ts"),
                        mode,
                    ],
                    cwd=envdir,
                    env=env,
                    stdout=out,
                    stderr=out,
                )
                processes.append(child)
                return child

            worker = launch("start", worker_env)
            wait(lambda: ready.is_set() or worker.poll() is not None)
            assert ready.is_set() and not errors, (
                errors,
                (case / "start.log").read_text(),
            )
            assert worker.poll() is None
            worker.kill()
            worker.wait(timeout=10)
            assert worker.returncode == -signal.SIGKILL
            release.set()
            thread.join(10)
            listener.close()
            assert not thread.is_alive()
            request, first_reply = captured[0]
            revision = first_reply["ok"]["revision"]

            def db_state():
                with sqlite3.connect(case / "controller.sqlite") as db:
                    return {
                        "effects": db.execute(
                            "SELECT count(*) FROM events WHERE kind='report_progress'"
                        ).fetchone()[0],
                        "occupied_slots": db.execute(
                            "SELECT count(*) FROM slots WHERE attempt IS NOT NULL"
                        ).fetchone()[0],
                    }

            before = db_state()
            assert before == {"effects": 1, "occupied_slots": 1}, before
            if revoked:
                command("cancel_attempt", {"attempt": grant["attempt"]})
            recovery = launch(
                "recover",
                {**worker_env, "PROBE_SOCKET": sock, "PROBE_REVISION": "999999"},
            )
            recovery.wait(timeout=30)
            assert recovery.returncode == 0, (case / "recover.log").read_text()
            records = [
                json.loads(x) for x in (case / "worker.jsonl").read_text().splitlines()
            ]
            calls = [x for x in records if x["phase"] == "before-effect"]
            replies = [x for x in records if x["phase"] == "reply"]
            assert len(calls) == 2 and len(replies) == 1, records
            settled = [x for x in records if x["phase"] == "settled"]
            assert len(settled) == 1
            tool_results = settled[0]["toolResults"]
            assert len(tool_results) == 1
            assert tool_results[0][0]["isError"] == revoked, tool_results
            assert settled[0]["status"] == "done"
            for key in ["taskId", "callId", "command", "commandHash"]:
                assert calls[0][key] == calls[1][key], key
            assert calls[0]["callId"] == "original-invocation"
            recovered = replies[0]["reply"]
            after = db_state()
            assert after == before
            if revoked:
                assert "error" in recovered and (
                    "revoked" in recovered["error"] or "stale" in recovered["error"]
                ), recovered
            else:
                assert recovered == first_reply, (recovered, first_reply)
            negative = None
            if not revoked:
                changed = json.loads(json.dumps(request))
                changed["command"]["args"]["text"] = "different payload"
                mismatch = rpc(sock, changed)
                assert "error" in mismatch and "idempotency" in mismatch["error"], (
                    mismatch
                )
                command(
                    "report_progress", request["command"]["args"], principal=worker_auth
                )
                duplicated = db_state()
                assert duplicated["effects"] == 2 and duplicated["occupied_slots"] == 1
                negative = {
                    "same_key_changed_payload": mismatch,
                    "fresh_key_current_revision": duplicated,
                }
            results.append(
                {
                    "scenario": "revoked" if revoked else "replay",
                    "worker_kill_returncode": worker.returncode,
                    "recovery_returncode": recovery.returncode,
                    "before_recovery": before,
                    "after_recovery": after,
                    "same_task_id": calls[0]["taskId"],
                    "same_invocation": calls[0]["callId"],
                    "command": calls[0]["command"],
                    "command_sha256": digest(calls[0]["command"]),
                    "committed_reply": first_reply,
                    "recovered_reply": recovered,
                    "committed_reply_sha256": digest(first_reply),
                    "recovered_reply_sha256": digest(recovered),
                    "pi_records": records,
                    "negative_control": negative,
                }
            )
            daemon.terminate()
            daemon.wait(timeout=10)
            log.close()
            os.close(fd)
            fds.remove(fd)
            # Private credential-bearing SQLite fixture is removed after retaining sanitized outcomes.
            shutil.rmtree(case)
    finally:
        for event in releases:
            event.set()
        for listener in sockets:
            try:
                listener.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass
            listener.close()
        for process in processes:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=10)
        for thread in threads:
            thread.join(15)
        for log in logs:
            log.close()
        for fd in fds:
            os.close(fd)
        for case in case_roots:
            if case.exists():
                shutil.rmtree(case)
        assert all(not thread.is_alive() for thread in threads), "proxy did not stop"
    result = {
        "upstream_commit": actual_pin,
        "runtime_source_manifest_sha256": digest(source_hashes),
        "runtime_source_files_verified": len(source_hashes),
        "node_version": subprocess.check_output(
            ["node", "--version"], text=True
        ).strip(),
        "dependency_lock_sha256": hashlib.sha256(
            (envdir / "package-lock.json").read_bytes()
        ).hexdigest(),
        "controller_base": CONTROLLER_PIN,
        "controller_source_manifest_sha256": digest(controller_hashes),
        "controller_source_files_verified": len(controller_hashes),
        "controller_binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "scenarios": results,
        "cleanup": {
            "all_proxy_threads_stopped": all(not t.is_alive() for t in threads),
            "private_case_directories_removed": all(not x.exists() for x in case_roots),
            "all_owned_processes_stopped": all(x.poll() is not None for x in processes),
            "process_returncodes": [x.returncode for x in processes],
        },
        "source_sha256": {
            f: hashlib.sha256((ROOT / f).read_bytes()).hexdigest()
            for f in [
                "experiments/pi-env-durable/effect-worker.ts",
                "experiments/pi-env-durable/interrupted-effect-probe.py",
            ]
        },
    }
    a.output.parent.mkdir(parents=True, exist_ok=True)
    a.output.write_text(json.dumps(result, indent=2) + "\n")
    print(
        json.dumps(
            {
                "passed": len(results),
                "output": str(a.output),
                "cleanup": result["cleanup"],
            }
        )
    )


if __name__ == "__main__":
    main()
