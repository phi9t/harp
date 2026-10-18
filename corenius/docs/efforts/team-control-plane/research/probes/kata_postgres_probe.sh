#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 2 ]]; then echo 'usage: kata_postgres_probe.sh PINNED_BINARY RESULTS_JSON' >&2; exit 64; fi
fixture="corenius-kata17-pg-$(date +%s)-$$"
image='postgres@sha256:f3bd19c606e442c3d7bdfa8002e03fe260a1023351e0ea4598032022b68dd6e3'
fixture_id="$(docker run --rm -d --name "$fixture" --label corenius.kata17=true -p 127.0.0.1::5432 --tmpfs /var/lib/postgresql/data:rw -e POSTGRES_USER=probe -e POSTGRES_DB=probe -e POSTGRES_HOST_AUTH_METHOD=trust "$image")"
trap 'docker stop "$fixture_id" >/dev/null' EXIT
ready=0
for i in $(seq 1 60); do
 if docker exec "$fixture_id" pg_isready -h 127.0.0.1 -p 5432 -U probe -d probe >/dev/null 2>&1; then ready=1; break; fi
 sleep 0.2
done
[[ "$ready" == 1 ]] || { echo 'fixture PostgreSQL readiness failed' >&2; exit 1; }
port="$(docker port "$fixture_id" 5432/tcp | sed 's/.*://')"
python3 "$(dirname "$0")/kata_remote_probe.py" --binary "$1" --output "$2" --pg-container "$fixture" --pg-port "$port"
