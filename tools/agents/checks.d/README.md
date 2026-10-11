One script per feature: `NN-name.sh`, run from the repository root by
`tools/agents/check.sh`. Exit non-zero on failure. Register a suite once; the
shell dispatcher is exercised by `05-gate-dispatch.sh` without running service
fixtures.

The default gate includes SQLite and excludes the parked PostgreSQL/Temporal
integration. `CHECK_TEAM_CONTROL_PLANE=1` enables that integration and requires
Temporal and Docker. `CHECK_CARGO=0` skips only the root Cargo suite; feature
Cargo suites still run.

Run every profile, including sibling policy drift and benchmark/capsule checks:

```sh
python3 tools/temp_root.py -- env CHECK_TEAM_CONTROL_PLANE=1 CHECK_BENCH=1 CHECK_SYNC=1 tools/agents/check.sh
```

The temporary-root behavior suite lives in `00-temp-paths.sh`; the shared source
scanner runs once in `90-no-host-tmp.sh`.
