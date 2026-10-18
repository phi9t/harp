#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
name="corenius-kernel-pg-$(date +%s)-$$"
image='postgres@sha256:f3bd19c606e442c3d7bdfa8002e03fe260a1023351e0ea4598032022b68dd6e3'
docker run --rm -d --name "$name" -p 127.0.0.1::5432 --tmpfs /var/lib/postgresql/data:rw -e POSTGRES_USER=probe -e POSTGRES_DB=probe -e POSTGRES_HOST_AUTH_METHOD=trust "$image" >/dev/null
pg_sampler_pid=''
port="$(docker port "$name" 5432/tcp | sed 's/.*://')"
postgres_tcp_ready() {
 python3 - "$port" <<'PY' >/dev/null 2>&1
import socket
import struct
import sys

def read_exact(sock, count):
    chunks = []
    remaining = count
    while remaining:
        chunk = sock.recv(remaining)
        if not chunk:
            raise OSError("short read")
        chunks.append(chunk)
        remaining -= len(chunk)
    return b"".join(chunks)

port = int(sys.argv[1])
with socket.create_connection(("127.0.0.1", port), timeout=0.5) as sock:
    sock.settimeout(0.5)
    params = b"user\0probe\0database\0probe\0application_name\0fixture-ready\0\0"
    sock.sendall(struct.pack("!II", len(params) + 8, 196608) + params)
    while True:
        tag = read_exact(sock, 1)
        size = struct.unpack("!I", read_exact(sock, 4))[0]
        payload = read_exact(sock, size - 4)
        if tag == b"E":
            raise SystemExit(1)
        if tag == b"R" and struct.unpack("!I", payload[:4])[0] != 0:
            raise SystemExit(1)
        if tag == b"Z":
            break
    sock.sendall(b"X" + struct.pack("!I", 4))
PY
}
cleanup() {
 if [[ -n "$pg_sampler_pid" ]]; then
  kill "$pg_sampler_pid" 2>/dev/null || true
  wait "$pg_sampler_pid" 2>/dev/null || true
 fi
 if [[ -n "${TEMPORAL_FIXTURE_EVIDENCE:-}" ]]; then
  docker logs "$name" >"$TEMPORAL_FIXTURE_EVIDENCE/postgres.log" 2>&1 || true
  python3 - "$port" "$TEMPORAL_FIXTURE_EVIDENCE/postgres-final.json" <<'PY' || true
import json
import socket
import sys

port = int(sys.argv[1])
path = sys.argv[2]
try:
    with socket.create_connection(("127.0.0.1", port), timeout=0.5):
        ready = True
except OSError:
    ready = False
with open(path, "w") as f:
    json.dump({"tcp_ready_before_container_stop": ready, "docker_exec": "skipped"}, f)
PY
 fi
 docker stop "$name" >/dev/null
 if [[ -n "${TEMPORAL_FIXTURE_EVIDENCE:-}" ]]; then
  python3 - "$name" "$TEMPORAL_FIXTURE_EVIDENCE/postgres-cleanup.json" <<'PYCODE'
import json
import subprocess
import sys

remaining = subprocess.run(
    ["docker", "ps", "-a", "--filter", f"name=^/{sys.argv[1]}$",
     "--format", "{{.Names}}"],
    check=True, capture_output=True, text=True,
).stdout.splitlines()
removed = sys.argv[1] not in remaining
with open(sys.argv[2], "w") as output:
    json.dump({"container": sys.argv[1], "owned_container_removed": removed}, output)
if not removed:
    raise SystemExit("owned PostgreSQL container remained after stop")
PYCODE
 fi
}
trap cleanup EXIT
if [[ -n "${TEMPORAL_FIXTURE_EVIDENCE:-}" ]]; then
 python3 - "$name" "$image" "$port" "$TEMPORAL_FIXTURE_EVIDENCE/postgres-fixture.json" <<'PYCODE'
import json
import sys

with open(sys.argv[4], "w") as output:
    json.dump({"container": sys.argv[1], "image": sys.argv[2],
               "host": "127.0.0.1", "port": int(sys.argv[3]),
               "storage": "disposable tmpfs", "authentication": "fixture-only trust"}, output)
PYCODE
fi
ready=0
for i in $(seq 1 60); do
 if postgres_tcp_ready; then ready=1; break; fi
 sleep 0.2
done
[[ "$ready" == 1 ]] || { echo 'PostgreSQL fixture failed readiness' >&2; exit 1; }
export TEAM_CONTROL_PLANE_TEST_DSN="host=127.0.0.1 port=$port user=probe dbname=probe"
if [[ -n "${TEMPORAL_FIXTURE_EVIDENCE:-}" ]]; then
 python3 - "$port" <<'PY' >"$TEMPORAL_FIXTURE_EVIDENCE/postgres-connections.jsonl" 2>&1 &
import json
import socket
import sys
import time

port = int(sys.argv[1])
while True:
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=0.5):
            ready = True
    except OSError:
        ready = False
    print(json.dumps({"at": time.time(), "tcp_ready": ready}), flush=True)
    time.sleep(1)
PY
 pg_sampler_pid=$!
fi
# Each assembled case owns several PostgreSQL clients and SDK workers. Bound
# independent fixtures on the shared test server; tests retain their own physical
# concurrency, and callers may explicitly choose a different libtest setting.
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-2}"
cargo test --locked "$@"
