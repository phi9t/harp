# Corenius

Corenius is a standalone Rust terminal agent prototype. It uses a turn-scoped
runtime, a Codex direct Responses transport, permissioned tools, and an
`insula` bubblewrap rootfs boundary.

## Local engineering team prototype

The retained SQLite controller, Kata/Git integrations, Codex runtime adapters and
MAC terminal views now live in this repository. See
[implementation status and code map](docs/efforts/sqlite-controller/IMPLEMENTATION-STATUS.md),
[controller usage](crates/sqlite-controller/README.md) and
[operator controls](docs/efforts/mac-operator-center/README.md).
The complete live two-worker baseline remains unqualified; see the status record
before treating a source landing as product acceptance.

For optional local ticket coordination, see [Collab operations](tools/collab/README.md).
Its Linux CLI provides Corenius and standalone ticket sources, worktrees, exact
checks, explicit operator landing, recovery, and archive-backed cleanup. The older
worktree and landing commands remain compatibility entry points. Adoption and
ledger migration remain operator decisions.

## Commands

- `cargo run -- run --workspace <path>`
- `cargo run -- run --workspace <path> --provider traecli`
- `cargo run -- eval aime --workspace <path> [--model <id>] [--stream]`
- `cargo run -- eval aime-sanity --workspace <path> [--stream]`
- `cargo run -- provider smoke traecli --workspace <path> [--stream]`
- `cargo run -- provider smoke traecli --workspace <path> --supervise`
- `cargo run -- provider supervise traecli --pid <pid>`
- `cargo run -- insula scan --store-root <path>`
- `cargo run -- insula select --rootfs <path> --store-root <path>`
- `cargo run -- insula doctor --dry-run`

The default Codex model is chosen from `SAND_CODEX_MODEL`, then Codex config,
then `gpt-5.4`.

The `traecli` provider starts `traecli acp serve` over stdio and uses the ACP
`initialize`, `session/new`, and `session/prompt` JSON-RPC flow. Set
`TRAECLI_PATH` to override the executable and pass `--model <id>` to pin a
TraeCode model.

Use `provider smoke traecli --supervise` to print `supervisor: <json>` process
snapshots while the ACP child is running. Use `provider supervise traecli --pid
<pid>` to inspect an existing TraeCLI process and its subprocess tree.

## Quick Eval

`eval aime` runs the full GAIR AIME benchmark through the TraeCLI ACP provider.
Each sample runs in an isolated ACP session: the user's TraeCLI plugins, skills,
memory, MCP servers, and tools are disabled so the model solves the problem with
reasoning alone. Pass `--inherit-user-config` to opt out, `--reasoning-effort
<level>` to pin the model's reasoning effort, `--idle-timeout-ms` to fail a
sample as stalled when no session update arrives (default 90s for `eval aime`),
and `--stall-retries` to control how often a stalled sample is retried (default
1). Partial response text is kept on timeouts and stalls, and `attempts` is
recorded per sample.
By default it evaluates all 60 problems from `2024`, `2025-I`, and `2025-II`,
fetching missing JSONL data into `.scratch/aime-benchmark/data/` and writing
`samples.jsonl`, `summary.json`, and `triage.md` under a timestamped
`.scratch/aime-benchmark/run-*` directory. Use `--years`, `--offset`,
`--limit`, and `--filter-id <year:id>` to narrow the run; use `--refresh-data`
to refetch the GAIR data cache.

`eval aime-sanity` runs a tracked five-case AIME sanity set from
`evals/aime_sanity.jsonl` through the TraeCLI ACP provider and writes
`samples.jsonl`, `summary.json`, and `triage.md` under `.scratch/aime-sanity/latest`
by default. The default run skips the known timeout probe and evaluates five
representative pass cases, including three easy cases; add
`--include-timeout-probe` when testing timeout observability.

Useful invocations:

- `TRAECLI_PATH=/data02/home/philip.yang/.local/bin/traecli cargo run -- eval aime --workspace /data02/home/philip.yang/workspace/corenius --model gpt-5.2 --years 2025-I --limit 1 --timeout-ms 120000 --reasoning-effort high --stream`
- `TRAECLI_PATH=/data02/home/philip.yang/.local/bin/traecli cargo run -- eval aime-sanity --workspace /data02/home/philip.yang/workspace/corenius --model gpt-5.2 --timeout-ms 300000 --stream`
- `cargo run -- eval aime-sanity --workspace . --filter-id 2025-I:I-1 --run-dir .scratch/aime-sanity/one`

The `run` command opens a ratatui/crossterm terminal UI. Type a prompt and press
Enter to start a turn, press `y` or `n` for approval prompts, and press Ctrl-C
or Esc to cancel and exit.

## Insula Contract

- Selection file: `.corenius/insula/selected.json`
- Rootfs marker: `.corenius-insula-rootfs.json`
- Marker fields: `schema_version: 1`, `owner: "corenius-insula"`, `digest`,
  `node_path`, and optional `dependency_lock_digest`
- The selected rootfs must live under `store_root`, must not be a symlink, and
  the saved digest must match the current marker digest when loaded.
