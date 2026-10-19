#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
name="corenius-kernel-pg-$(date +%s)-$$"
image='postgres@sha256:f3bd19c606e442c3d7bdfa8002e03fe260a1023351e0ea4598032022b68dd6e3'
docker run --rm -d --name "$name" -p 127.0.0.1::5432 --tmpfs /var/lib/postgresql/data:rw -e POSTGRES_USER=probe -e POSTGRES_DB=probe -e POSTGRES_HOST_AUTH_METHOD=trust "$image" >/dev/null
trap 'docker stop "$name" >/dev/null' EXIT
ready=0
for i in $(seq 1 60); do
 if docker exec "$name" pg_isready -h 127.0.0.1 -p 5432 -U probe -d probe >/dev/null 2>&1; then ready=1; break; fi
 sleep 0.2
done
[[ "$ready" == 1 ]] || { echo 'PostgreSQL fixture failed readiness' >&2; exit 1; }
port="$(docker port "$name" 5432/tcp | sed 's/.*://')"
export TEAM_CONTROL_PLANE_TEST_DSN="host=127.0.0.1 port=$port user=probe dbname=probe"
cargo test --locked "$@"
