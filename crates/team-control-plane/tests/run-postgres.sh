#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
name="corenius-kernel-pg-$(date +%s)-$$"
image='postgres@sha256:f3bd19c606e442c3d7bdfa8002e03fe260a1023351e0ea4598032022b68dd6e3'
docker run --rm -d --name "$name" -p 127.0.0.1::5432 --tmpfs /var/lib/postgresql/data:rw -e POSTGRES_USER=probe -e POSTGRES_DB=probe -e POSTGRES_HOST_AUTH_METHOD=trust "$image" >/dev/null
pg_sampler_pid=''
cleanup() {
 if [[ -n "$pg_sampler_pid" ]]; then
  kill "$pg_sampler_pid" 2>/dev/null || true
  wait "$pg_sampler_pid" 2>/dev/null || true
 fi
 if [[ -n "${TEMPORAL_FIXTURE_EVIDENCE:-}" ]]; then
  docker logs "$name" >"$TEMPORAL_FIXTURE_EVIDENCE/postgres.log" 2>&1 || true
  docker exec "$name" psql -U probe -d probe -Atc "SELECT json_build_object('max_connections',current_setting('max_connections'),'connections',count(*)) FROM pg_stat_activity" >"$TEMPORAL_FIXTURE_EVIDENCE/postgres-final.json" || true
 fi
 docker stop "$name" >/dev/null
}
trap cleanup EXIT
ready=0
for i in $(seq 1 60); do
 if docker exec "$name" pg_isready -h 127.0.0.1 -p 5432 -U probe -d probe >/dev/null 2>&1; then ready=1; break; fi
 sleep 0.2
done
[[ "$ready" == 1 ]] || { echo 'PostgreSQL fixture failed readiness' >&2; exit 1; }
port="$(docker port "$name" 5432/tcp | sed 's/.*://')"
export TEAM_CONTROL_PLANE_TEST_DSN="host=127.0.0.1 port=$port user=probe dbname=probe"
if [[ -n "${TEMPORAL_FIXTURE_EVIDENCE:-}" ]]; then
 (
  while true; do
   docker exec "$name" psql -U probe -d probe -Atc "SELECT json_build_object('at',clock_timestamp(),'max_connections',current_setting('max_connections'),'connections',count(*)) FROM pg_stat_activity" || true
   sleep 1
  done
 ) >"$TEMPORAL_FIXTURE_EVIDENCE/postgres-connections.jsonl" 2>&1 &
 pg_sampler_pid=$!
fi
# Each assembled case owns several PostgreSQL clients and SDK workers. Bound
# independent fixtures on the shared test server; tests retain their own physical
# concurrency, and callers may explicitly choose a different libtest setting.
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-2}"
cargo test --locked "$@"
