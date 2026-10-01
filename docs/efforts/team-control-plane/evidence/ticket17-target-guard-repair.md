# Ticket17 target guard repair2

Read Corenius base `963f58b31e518a8162dfe719e386ef9d9843321f` and frozen
repair1 manifest. Separate Standards review identified an inherited unsafe
public PG probe target guard. This narrow primary integration repair does not
change broker mapping, production code, adoption authority or runtime budget.

## Actual red/green and identity checks

A real disposable loopback tmpfs PostgreSQL fixture intentionally omitted the
ownership label. Original public `kata_remote_probe.py` completed and restored
its `probe` database (exit0); the assertion requiring rejection failed (exit1).
Only this deliberately owned negative-test fixture was affected and removed.
After the repair, the same negative public fixture rejects before daemon startup
(exit1), and the regression harness passes (exit0).

The guard resolves Docker inspection to an immutable container ID and requires
`corenius.kata17=true`, running state, no host bind mounts, a tmpfs PostgreSQL
data directory and exactly the measured DSN's `127.0.0.1` port. Restore checks
this identity again immediately before dump and immediately before deletion;
all database operations target the captured ID. The wrapper also stops its
captured ID rather than a potentially rebound name.

A separate real Docker fixture accepted its actual identity and rejected both
a different bound port and a different expected immutable ID (exit0). No fake
Docker client or mirrored database implementation was used. Fresh positive
public PG probe completed API/claim/credential checks and actual logical restore
(exit0), preserving closed issue, winning claim and expired-token denial.
Processes/listeners were stopped; all four test containers were removed through
checked cleanup. Bash syntax and Python compilation passed. Zero model calls.

## Reproduction and retained evidence

Positive: `bash research/probes/kata_postgres_probe.sh PINNED_BINARY RESULTS_JSON`
(paths are relative to this effort directory). Negative: create the same owned
loopback/tmpfs image fixture without its label, then call the public remote probe
with its name/port; require rejection before any daemon or database mutation.
To test identity drift, invoke its `owned_postgres` function against a real
labeled fixture with a mismatched port or expected ID; both must reject.

Fresh positive observations are in `ticket17-kata-postgres-results.json`.
Run log digests and exact observed exit codes are retained in
`ticket17-target-guard-results.json`. Prior immutable broker mapping/replay
results are unchanged. Separate Standards and Spec re-review remain required;
ticket17 is still unaccepted and the proposed defer disposition is unchanged.
