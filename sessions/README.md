# harp-session-index

Private **operator analysis** indexer for TraeCLI **Execution sessions**.

Reads raw **Rollout artifacts** (JSONL), not AgentsView archives. v1 supports
Trace dialect `traecli.dual_stream.v1` (`event_msg` ∪ `history_mutation`).

See:

- `docs/adr/0003-session-analyzer-raw-rollouts-pluggable-dialects.md`
- `knowledge/investigations/local/long-horizon-trace-analysis/20260904-traecli-long-horizon/dialects/traecli-dual-stream/SPEC.md`

## CLI

```sh
cargo run -p harp-session-index -- detect path/to/rollout.jsonl
cargo run -p harp-session-index -- index path/to/rollout.jsonl \
  --index-root "$XDG_DATA_HOME/harp/session-index" \
  --artifacts-dir path/to/rollout.artifacts/tool-results
cargo run -p harp-session-index -- show <session-id> --index-root ...
cargo run -p harp-session-index -- verify path/to/rollout.jsonl <session-id> \
  --artifacts-dir path/to/rollout.artifacts/tool-results
```

`verify` re-indexes into a scratch directory, diffs the claimed `summary.json`,
writes `verify-report.json`, and prints structured feedback. Exit code `2`
means field mismatches.

Indexes and oversize spills are written under the index root (default
`$XDG_DATA_HOME/harp/session-index` or `~/.local/share/harp/session-index`).

## Tests

```sh
cargo test -p harp-session-index
```
