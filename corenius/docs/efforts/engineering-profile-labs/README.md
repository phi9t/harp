> Historical record: the PostgreSQL/Temporal lab runner and its experiment
> have been retired from the active payload. Their original source is retained
> in Git ancestry and declared in `corenius/HARP_IMPORT.json`. The commands
> below describe the historical qualification; they are not current setup
> instructions. Native provider records and captured evidence remain intact.

# Engineering planner and runtime learning labs

These five labs explain the components needed to execute a skill-based plan
through collaborating agents. They reuse Corenius source at
`b7bee20e00c822feec48362a1dbd8d62fe9c3641` and record actual local observations.
This is learning evidence before service adoption into Trondheim.

| Lab | Learn | Observed result |
| --- | --- | --- |
| [PostgreSQL](POSTGRESQL.md) | Claims, receipts, revision races, atomic admission and fencing | 16 native tests passed |
| [Temporal](TEMPORAL.md) | Durable waits, worker replacement, retries, rollover and replay | 7 native tests passed; 11 explicit SDK replays |
| [TraeCLI / ACP](native/01-traecli.md) | Process events, ACP handshake, model catalog and terminal outcomes | Offline initialization passed; session creation required authentication; two live turns failed |
| [Codex CLI](native/02-codex-cli.md) | JSON event stream, process exit and turn acceptance | One live arithmetic turn completed |
| [Codex app-server](native/03-codex-app-server.md) | RPC, thread/turn identity, notifications and process lifetime | One live completion observed; empty-thread resume passed; historical tool-absence evidence is incomplete |

The app-server projection joins the answer to the completed turn, but omitted
item categories needed by the stricter current tool guard. It cannot be
requalified under that guard from the retained evidence alone.

PostgreSQL owns canonical transitions and authority. Temporal retains workflow
history and schedules observations. A CLI or app-server performs an admitted
model turn. A saved conversation does not replace the controller's job record.

Read PostgreSQL first, then Temporal and the [native runtime index](native/README.md).
The experiment contract is in [SPEC.md](SPEC.md). The fresh result and proof
limits are in [evidence/summary.md](evidence/summary.md).

## Database lab prerequisites

Use Linux with Docker available, Rust/Cargo 1.95 and Temporal CLI 1.7.0.
The local cache must contain the locked Cargo dependencies and pinned image:

```text
postgres@sha256:f3bd19c606e442c3d7bdfa8002e03fe260a1023351e0ea4598032022b68dd6e3
```

The runner fails if required native dependencies are absent. Provision missing
dependencies separately. It never substitutes a fake database or fake server.
Keep `TMPDIR` in an owned absolute directory outside `/tmp`.

## Run both database labs

From this clone's root:

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/run.py --lab all --timeout-seconds 1200
```

Use `--lab postgres` or `--lab temporal` for one system. The Temporal lab starts
its own PostgreSQL fixture; it does not reuse the PostgreSQL lab database.

The runner prints the owned scratch evidence directory. Each lab retains
`test.log`, `result.json`, fixture details and cleanup files. Temporal also
retains actual SDK histories, retry observations and a local dev-server database.
The run-level `metadata.json` records source inputs and versions.
The run-level `summary.json` is accepted only if all selected labs pass.

Compilation resolves Cargo dependencies offline. This does not establish host
network isolation. The dev server listens on loopback; no shared service is used.

## Read a result

1. Open `metadata.json` and verify the source and input digests.
2. Open each `result.json` and verify `accepted: true` and exit code zero.
3. Check the required tests are present and the suite has no failures.
4. Check `postgres-cleanup.json` names the removed owned container.
5. For Temporal, check `cleanup.json` reports its listener closed.
6. Inspect the recorded history files using the walkthrough below.
7. Preserve a failed run and its reason before any retry.

A zero exit is a bounded behavior result, not a production-readiness decision.
Do not promote these experiments into Trondheim on the basis of one passing run.

## Run the native labs

Start with model-free inventory, schema and protocol probes in
[native/README.md](native/README.md). Those commands and parser tests make no
provider request. The guides also document opt-in live modes with an explicit
spend flag. Their time limit is not a hard token or monetary ceiling.

## Adoption assessment

The experiment supports adopting PostgreSQL claims incrementally: the parked
kernel already supplies testable admission, idempotency and fencing behavior.
Deployment, migration ownership and operational recovery still need a separate
decision. Temporal recovery needs more integration work around native turns:
a retry after a lost acknowledgment can duplicate an external effect. Persist
intent and exact native identities, then reconcile uncertain outcomes before
dispatching again.

The Trondheim profile currently compiles pinned plan proposals and projects a
ready frontier. It grants no execution authority. These labs do not install a
second scheduler or qualify a production coordinator. Active-turn recovery,
lost-ack reconciliation and enforced provider budgets remain adoption gates.

## Next steps

The [Corenius adoption plan](../engineering-profile-adoption/README.md) preserves
the original design verbatim and defines the decision and implementation frontier.
Its first ticket reconciles the retained I2/Kata priority with proposed
PostgreSQL/Temporal adoption before migration or new provider execution.
