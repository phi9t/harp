#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
evidence="${TEMPORAL_FIXTURE_EVIDENCE:-$(mktemp -d /tmp/corenius-temporal13.XXXXXX)}"
mkdir -p "$evidence"
export TEMPORAL_FIXTURE_EVIDENCE="$evidence"
port="$(python3 - <<'PY'
import socket
s=socket.socket();s.bind(('127.0.0.1',0));print(s.getsockname()[1]);s.close()
PY
)"
temporal server start-dev --headless --ip 127.0.0.1 --port "$port" --db-filename "$evidence/temporal.sqlite" >"$evidence/server.log" 2>&1 &
server_pid=$!
cleanup() {
 kill "$server_pid" 2>/dev/null || true
 wait "$server_pid" 2>/dev/null || true
 python3 - "$port" "$evidence/cleanup.json" <<'PYCODE'
import json,socket,sys
s=socket.socket();s.settimeout(.5)
closed=s.connect_ex(('127.0.0.1',int(sys.argv[1])))!=0;s.close()
with open(sys.argv[2],'w') as f: json.dump({'temporal_listener_closed':closed},f)
if not closed: raise SystemExit('owned Temporal listener remained open')
PYCODE
}
trap cleanup EXIT
export TEMPORAL_ADDRESS="127.0.0.1:$port"
ready=0
for _ in $(seq 1 100); do
 if temporal operator namespace describe --namespace default --address "$TEMPORAL_ADDRESS" >/dev/null 2>&1; then ready=1; break; fi
 kill -0 "$server_pid" || { cat "$evidence/server.log"; exit 1; }
 sleep 0.1
done
[[ "$ready" == 1 ]] || { echo 'Temporal failed readiness' >&2; exit 1; }
./tests/run-postgres.sh "$@"
