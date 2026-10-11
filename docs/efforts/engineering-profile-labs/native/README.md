# Native runtime learning labs

These labs explain the native runtime boundary before Trondheim adopts a
persistent engineering control plane. They add learning tools and evidence;
they do not select a production architecture or change a runtime adapter.

Source revision read: `b7bee20e00c822feec48362a1dbd8d62fe9c3641`.
Observed on 2026-10-11. Installed runtime versions are TraeCLI 0.209.1,
Codex CLI 0.160.1, and Temporal CLI 1.7.0. Binary digests and public help
output are in [inventory](evidence/20261011-inventory.json).

## Read and run

Follow [TraeCLI and ACP](01-traecli.md), [Codex CLI](02-codex-cli.md), then
[Codex app-server](03-codex-app-server.md). Read these beside the effort's
PostgreSQL and Temporal labs. PostgreSQL stores controller facts; Temporal
executes admitted workflow transitions; a native runtime performs model turns.
These responsibilities remain separate.

Run commands from the Corenius repository root. Keep replay evidence under
`.scratch/engineering-profile-labs/native/`. The helper refuses to replace an
existing output file. Choose a fresh file for each replay.

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/native/run_lab.py inventory --out .scratch/engineering-profile-labs/native/inventory.json
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/native/run_lab.py schema --out .scratch/engineering-profile-labs/native/schema.json
python3 tools/temp_root.py -- python3 -m unittest discover -s docs/efforts/engineering-profile-labs/native -p 'test_*.py' -v
```

The model-free Insula probes require Linux, `bwrap`, working user namespaces,
the installed native binary, and the host's `/usr` closure. They adapt the
existing `crates/sqlite-controller/tests/support/native_init.py` boundary.
They unshare namespaces, clear the environment, expose no credential store,
and allocate private sandbox temporary files at `/run/corenius-tmp`.
Their `/usr` input is an ambient read-only dependency closure; these probes
are network-isolated and disposable, but do not establish reproducible,
content-pinned hermetic execution.

Live modes require an existing authenticated **trusted installed runtime**.
The helper never reads credentials, emits bearer material, changes global
configuration, or supplies credentials to fake fixtures. Live modes run on the
host with read-only tool policy; they are explicitly outside Insula. The
app-server live mode inherits ambient runtime configuration. Its RPC evidence
uses an allowlist before persistence to exclude account and configuration data.

Each live command requests one arithmetic turn and has a 1–60 second deadline.
No native ordinary-turn hard token or monetary ceiling was established.
`--authorize-model-spend` admits a bounded-time learning attempt, not a billing
guarantee. Tiny prompts can still have large harness input: the Codex CLI
attempt used 13,822 input tokens and 5 output tokens. Production admission
requires independently enforced provider spend limits.

## Evidence classes and fresh results

| Evidence class | Actual action | Result and limit |
| --- | --- | --- |
| `INSTALLED_BINARY_READ_ONLY` | Native version/help commands | Installed interfaces, not runtime qualification |
| `MODEL_FREE_ACTUAL_SCHEMA` | Native schema generation in Insula | 440 actual schemas; schema presence alone does not prove implementation |
| `MODEL_FREE_ACTUAL_PROTOCOL` | Actual offline ACP/app-server RPCs | ACP initialize succeeds, session creation requires authentication; app-server stores/resumes an empty thread across process replacement |
| `MODEL_FREE_NATIVE_CATALOG` | Native Trae model catalog | Advertised model names, not successful inference entitlement |
| `REAL_PROVIDER_ATTEMPT` | Trusted native runtime arithmetic turn | Codex CLI and app-server passed; both TraeCLI attempts failed |
| `FAKE_CONFORMANCE_FIXTURE` | Unit-test messages | Parser/acceptance behavior only; never substitutes for native completion |

[Codex CLI evidence](evidence/20261011-codex-cli.json) joins process exit,
thread identity, completed answer, and `turn.completed`.
[App-server provider evidence](evidence/20261011-codex-app-server-live.json)
joins thread ID, turn ID, completed answer, and `turn/completed`; the process
remains alive afterward. [TraeCLI evidence](evidence/20261011-traecli-catalog-model.json)
contains `turn.failed`, exit code 1, and no accepted completion.

## What a controller must own

| Controller concept | Native counterpart | Consequence |
| --- | --- | --- |
| `ExecutionScope` | None established | Persist jobs, grants, budgets, and outstanding effects outside the runtime |
| `Episode` | One admitted turn or bounded CLI attempt | Associate the outcome with intent and acceptance evidence |
| `RuntimeIncarnation` | One service process lifetime | Track independently from persistent native thread identity |
| Native thread/session | Conversation store key | Save the exact key; do not replace a lost acknowledgment with a new conversation |
| Native turn | App-server turn ID or CLI event interval | Join terminal events to the admitted turn, not merely to an alive PID |
| `Quarantine` | No automatic native equivalent | Preserve ambiguous in-flight operations until effects are reconciled |

Inference: use native runtimes as supervised Episode executors. Their saved
conversation context does not own controller authority or durable jobs. A
Temporal retry after a lost `turn/start` acknowledgment can duplicate work.
Record intent before dispatch, retain unknown outcomes, and reconcile exact
native identities before admitting another turn.

## Cleanup and qualification gaps

The helper terminates its owned process group, waits for disposal, and removes
temporary working directories. Model-free stores and generated schemas are
deleted. Live CLI attempts are ephemeral; the app-server live thread is also
ephemeral. Credential stores remain owned by the trusted native runtime.
The lab does not claim cleanup of pre-existing shared runtime services.

Active-turn interruption, native provider retries, lost-ack reconciliation,
process replacement during a tool call, stale controller fencing, shared
daemon ownership, and production spend enforcement remain unqualified. The
model-free persisted empty-thread result proves conversation persistence only.

The reviewed reducer requires one ordered interval: thread identity, then
`turn.started`, then the completed arithmetic answer, then `turn.completed`.
Every supplied thread identity must match. A shared allowlist rejects unknown
item types for both native interfaces. Temporary-root validation reuses
`tools/temp_root.py`, including resolved paths, ownership, and write permissions.

The final suite passes 21 model-free tests. Repair red runs exposed eight
ordering/identity failures, five unknown-item failures, and three unsafe
temporary-root failures before the corresponding fixes passed. A retained
evidence test re-reduces three actual native CLI artifacts: Codex remains
accepted; both Trae attempts remain rejected. No runtime/provider was invoked.

Historical provider evidence was captured before these final guard changes.
The app-server projection omitted item types, so the current conservative tool
guard cannot qualify that old projection as accepted. Its joined arithmetic
completion and process-alive observation remain actual evidence. Additional
tool classification is an explicit gap; historical evidence is unchanged.

Workflow route: bounded learning work under the approved effort; no product
API/schema or architecture mutation. The unavailable Matt routing commands
were handled with the repository crosswalk. The clean-context implementation
slice uses a private `--no-local` clone. No commit, push, merge, deployment,
or downstream adoption is authorized by these files.
