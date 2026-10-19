# PostgreSQL fixture feasibility

Status: completed bounded feasibility assessment; not ticket 10 acceptance. No product implementation or owner decision.
Source commit read: `963f58b31e518a8162dfe719e386ef9d9843321f`.
Planning inputs read: SPEC.md, PLAN.md and ticket 10 from the reference-design worktree on 2026-10-01.

Scope: uniquely named disposable loopback-only PostgreSQL container, scratch Rust client, actual rollback/uniqueness/capacity-lock probes. No shared service changes, broker implementation, model calls, commits, pushes or decisions. Cleanup required after probes. Results below are fixture observations rather than ticket 10 acceptance.

## Verified fixture observations

Run date: 2026-10-01. Docker server 26.1.4; rustc 1.95.0
(59807616e 2026-04-14); cargo 1.95.0 (f2d3ce0bd 2026-03-21).

- Pulled `postgres:17.6-bookworm`, then launched by immutable manifest digest
  `postgres@sha256:f3bd19c606e442c3d7bdfa8002e03fe260a1023351e0ea4598032022b68dd6e3`.
  Local amd64 image ID: `sha256:567d5d704a3e5a4864393b623a0d50eb1b5a4ad134cdac7cf19555cb2d8ffe9e`.
- Server reported PostgreSQL 17.6 (Debian 17.6-2.pgdg12+1), x86_64,
  GCC Debian 12.2.0-14+deb12u1, 64-bit.
- Port inspection reported `5432/tcp -> 127.0.0.1:10000`; Docker inspect
  confirmed HostIp 127.0.0.1 and data tmpfs `/var/lib/postgresql/data:rw`.
  No host data volume or shared service was modified.
- Scratch client pinned `tokio-postgres = "=0.7.18"` and
  `tokio = "=1.52.3"`; build succeeded in 8.15s. Local downloaded crate
  manifest declares Rust 1.85 minimum and Tokio 1.27 dependency range.
  Corenius base manifest declares Rust 1.95 and Tokio 1. This is compatible
  at the declared version/interface level; no product Cargo.toml was changed.
- Actual TCP client connected using `NoTls`, with its connection future spawned.
- Explicit rollback after inserts to three fixture tables left total persisted
  rows **0**. Dropping an uncommitted transaction also left rows **0**.
- Duplicate primary key returned PostgreSQL SQLSTATE **23505**.
- While connection A held `SELECT ... FOR UPDATE`, connection B's same lock
  failed after its 150ms lock timeout with SQLSTATE **55P03**. A committed
  `used=1`; B's next transaction acquired the lock and observed **1**.
  The fixture therefore demonstrated serialization and visible exhausted
  capacity, without simulating SQL state in memory.
- `cargo run --locked` exited 0 with all assertions passing. Lockfile SHA256:
  `315db690c16126bdbcb0cba7bdb5c65100b4179a39b73b57333ba6af173ebce1`.
  Probe source SHA256:
  `d8e196f04a95bf0ab458374d15e15ed0eaadb7612361a3a704835bf891605e57`.
- `docker stop` succeeded; subsequent exact-name `docker ps -a` returned no
  container. `--rm` removed the container and ephemeral database. Downloaded
  image and gitignored build files remain available; no secrets were created.

## Interpretation and unverified scope

Inference: this host can run a real isolated PostgreSQL behavioral fixture,
using a Rust async client compatible with the declared Corenius toolchain.
`tokio-postgres` is a tested candidate, not an owner-approved product choice.
The capacity probe establishes lock behavior; it is not the broker's actual
admission policy. Ticket 10 remains blocked by its owner decisions.

Not verified: full Corenius dependency integration, SQLx/Diesel alternatives,
connection pooling, TLS/password/role authentication, migrations, recovery
epoch or revision checks, idempotency response records, concurrent duplicate
admission, dependency cycles, crash/restart durability, WAL/fsync or backups,
Temporal, gateway fencing, or live provider behavior. The tmpfs fixture is
intentionally ephemeral and cannot establish persistent-use durability.

Primary sources: [pinned client source and manifest](https://docs.rs/crate/tokio-postgres/0.7.18/source/),
[transaction API](https://docs.rs/tokio-postgres/0.7.18/tokio_postgres/struct.Transaction.html),
[PostgreSQL row locking](https://www.postgresql.org/docs/17/explicit-locking.html),
[PostgreSQL error codes](https://www.postgresql.org/docs/17/errcodes-appendix.html),
[official PostgreSQL container recipe](https://github.com/docker-library/postgres).
The browser retrieved PostgreSQL locking documentation; pinned docs.rs rendered
pages were unavailable through the browser, so crate API/manifest claims above
were checked against Cargo's downloaded primary source instead.

## Reproduce the isolated fixture

Use a new empty scratch crate directory. Save the manifest and source below.
The trust setting is for a disposable loopback fixture only. Choose a unique
container name; Docker allocates the loopback port. Always stop it afterward.

```bash
fixture_name="corenius-pg-probe-$(date +%s)-$$"
image='postgres@sha256:f3bd19c606e442c3d7bdfa8002e03fe260a1023351e0ea4598032022b68dd6e3'
docker pull "$image"
docker run -d --rm --name "$fixture_name" -p 127.0.0.1::5432 \
  --tmpfs /var/lib/postgresql/data:rw \
  -e POSTGRES_USER=probe -e POSTGRES_DB=probe \
  -e POSTGRES_HOST_AUTH_METHOD=trust "$image"
trap 'docker stop "$fixture_name" >/dev/null' EXIT
until docker exec "$fixture_name" pg_isready -U probe -d probe; do sleep 1; done
PGPROBE_PORT="$(docker port "$fixture_name" 5432/tcp | sed 's/.*://')" \
  cargo run --manifest-path Cargo.toml
# Retain Cargo.lock; repeat with --locked after initial resolution.
```

`Cargo.toml`:

```toml
[package]
name = "pg-feasibility-probe"
version = "0.0.0"
edition = "2021"
[workspace]
[dependencies]
tokio = { version = "=1.52.3", features = ["macros", "rt-multi-thread", "time"] }
tokio-postgres = "=0.7.18"
```

`src/main.rs`:

```rust
use tokio_postgres::{NoTls, error::SqlState};
async fn connect() -> tokio_postgres::Client {
 let port = std::env::var("PGPROBE_PORT").unwrap();
 let (c, connection) = tokio_postgres::connect(&format!("host=127.0.0.1 port={port} user=probe dbname=probe"), NoTls).await.unwrap();
 tokio::spawn(async move { connection.await.unwrap(); }); c
}
#[tokio::main]
async fn main() {
 let mut a = connect().await; let mut b = connect().await;
 let version: String = a.query_one("SELECT version()", &[]).await.unwrap().get(0); println!("{version}");
 a.batch_execute("CREATE TABLE command(key text PRIMARY KEY); CREATE TABLE attempt(id int PRIMARY KEY); CREATE TABLE outbox(id int PRIMARY KEY); CREATE TABLE capacity(id int PRIMARY KEY, used int NOT NULL CHECK (used BETWEEN 0 AND 1)); INSERT INTO capacity VALUES (1,0);").await.unwrap();
 let t=a.transaction().await.unwrap(); t.batch_execute("INSERT INTO command VALUES ('rollback'); INSERT INTO attempt VALUES (1); INSERT INTO outbox VALUES (1)").await.unwrap(); t.rollback().await.unwrap();
 let count: i64=a.query_one("SELECT (SELECT count(*) FROM command)+(SELECT count(*) FROM attempt)+(SELECT count(*) FROM outbox)", &[]).await.unwrap().get(0); assert_eq!(count,0); println!("explicit rollback: total persisted rows={count}");
 { let t=a.transaction().await.unwrap(); t.execute("INSERT INTO command VALUES ('drop')", &[]).await.unwrap(); }
 let count:i64=a.query_one("SELECT count(*) FROM command",&[]).await.unwrap().get(0); assert_eq!(count,0); println!("dropped transaction rollback: rows={count}");
 a.execute("INSERT INTO command VALUES ('same')", &[]).await.unwrap();
 let t=b.transaction().await.unwrap(); let e=t.execute("INSERT INTO command VALUES ('same')", &[]).await.unwrap_err(); assert_eq!(e.code(),Some(&SqlState::UNIQUE_VIOLATION)); t.rollback().await.unwrap(); println!("unique collision SQLSTATE=23505");
 let t=a.transaction().await.unwrap(); t.query_one("SELECT used FROM capacity WHERE id=1 FOR UPDATE", &[]).await.unwrap();
 b.batch_execute("SET lock_timeout='150ms'").await.unwrap();
 let e=b.query_one("SELECT used FROM capacity WHERE id=1 FOR UPDATE", &[]).await.unwrap_err(); assert_eq!(e.code(),Some(&SqlState::LOCK_NOT_AVAILABLE)); println!("competing row lock SQLSTATE=55P03");
 t.execute("UPDATE capacity SET used=1 WHERE id=1", &[]).await.unwrap(); t.commit().await.unwrap();
 let t=b.transaction().await.unwrap(); let used:i32=t.query_one("SELECT used FROM capacity WHERE id=1 FOR UPDATE",&[]).await.unwrap().get(0); assert_eq!(used,1); t.rollback().await.unwrap(); println!("after release competing admission sees used=1; no second reservation");
}
```
