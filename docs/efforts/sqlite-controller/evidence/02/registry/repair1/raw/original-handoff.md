# Ticket02 owner-amended native registry candidate

Sole implementation writer: `sqlite02/runtime` in
`/data02/home/philip.yang/workspace/corenius-sqlite02-runtime`. Coordination base
read/fast-forwarded: `a75c520761e5f93f078abdfd1d99f24369499b99`.
Tested source commit: `f77df6493f1090841bafa388b8b38b624a24571f` (after implementation `29506cf`).
Process core remains the independently qualified `77c4168` ancestry. This phase
implements the owner4dfe registry/topology amendment, not another process-core
repair. Root owns review, acceptance and integration. No SPEC/STATE/ticket edits,
source landing, shared branch rewrite, other checkout writes, tmux changes,
upstream/provider credential use or paid/live inference occurred.

Classification: **MODEL_FREE_ACTUAL_PROTOCOL**. All runtime receipts and lifecycle
observations come from the actual pinned native processes and Linux kernel.
A local Unix Responses/SSE provider supplies deterministic bytes; no model
inference takes place. This is not assembled-baseline/live qualification.

## Implementation and authority

The supported configured daemon autonomously drains the process runtime and
Codex adapter. Clients only issue ordinary team admission/cancellation/native
input commands. The fixture supplies protocol bytes, observes state and injects
faults in its own resources; it does not schedule through a second dispatcher.
External RPC/process effects occur between short SQLite journal transactions.

Explicit opt-in v1→v2 creates durable configuration, server, actor-binding and
native-operation tables. Configuration identity is retained and compared on
restart. Missing configuration for a nonempty configured registry is refused;
empty-profile schema diagnostics remain explicitly unqualified. The operator
projection exposes auth references, never plaintext native capabilities.

Bindings retain server ID, endpoint/auth reference, native and supervisor
incarnations, boot/process/start/namespace identities, protected store UUID and
filesystem identity, exact binary/companion/config/catalog/launcher/schema pins,
capability contract, owned execution handle, workspace identity/generation,
thread ID, native turn ID and typed active/terminal/unknown observation, actor
role/authority epoch, controlplane cwd and selected remote environment cwd/roots.
Only observations bearing that exact thread identity enter its activity projection.
Native inner dynamic call IDs are preserved within typed activity; no outer
model code-mode ID is substituted for them. Ticket05 owns actual tool authority.

Shared topology uses one independently supervised sterile app-server/store/
companion namespace with private executors. Separate topology uses a distinct
sterile app-server/store/companion namespace paired with each distinct executor.
Both components have independently qualified process handles and namespace
identities. The host `ptrace_scope` was0 and was not changed; placing shared app
capabilities alongside worker tools would weaken authority. Root explicitly
ruled this paired separate topology sufficient; no host ptrace workaround was used.

Sealed read-only descriptors mount pinned assets. No canonical checkout,
controller DB/socket, supervisor records/socket, host home/auth/SSH agent or
peer workspace is mounted. Private network interfaces reach fixed own-executor
and scoped provider Unix bridges, not arbitrary TCP/CONNECT. Read-only socket
mounts are capabilities and are confined accordingly. App-server bearer/auth
bridges are controller-only, outside worker/lead tool PID and filesystem realms.

A transient bearer exists inside its authenticated native executor's request
memory and may be visible within that own tool realm. It grants that already
isolated executor capability, not host/upstream credentials, shared app control,
peer effects or controller submission/acceptance authority. This proof does not
claim zero bearer bytes in native executors. Runtime cancellation withdraws
submission authority and destroys the qualified executor namespace independently.

Native mutations journal intent before RPC and record acknowledgement or unknown.
Every turn/start supplies the validated remote environment and fixed controller
permission policy. Native request IDs are not deduplication keys. Lost turn-start
acknowledgement reconciles exactly one actual typed user-message clientId and
input; unknown/applying starts fence new submissions and prepared new dispatch.
No arbitrary server/thread/history/path/environment/workspace or terminal input
is accepted. Actor and issuer authority are rechecked immediately before RPC.
Old lead grants and retired attempt bindings fail; lead role consumes no worker slot.

Controller restart adopts retained receipts and reconnects ID-only subscriptions.
Native server restart keeps the store but loses environment registration; ID-only
resume selects local and read-only defaults. The adapter restores its fixed policy,
re-registers the qualified executor and keeps UI eligibility blocked until an
explicit remote turn and typed observations confirm the actual environment.
Actual shell and apply_patch effects after restoration prove the intended realm.

Native thread/start creates a loaded identity but no resumable rollout until the
first turn. A controller crash after committing that receipt adopts the exact
loaded thread only in the same surviving qualified app incarnation, without a
second creation. Changed-incarnation unmaterialized creation, missing creation
acknowledgement and identity mismatch remain unknown with no blind recreation.

Cancellation first withdraws UI/input authority, then uses the retained old
thread/turn binding only for cleanup. Native typed completed/interrupted/failed
receipt, interrupt acknowledgement and unknown are explicit dispositions. Executor
pidfd termination and capacity release are separate observations. Task success is
never inferred from native text, native turn completion or process termination.

The native WebSocket implementation uses an eight-second absolute RPC send/read
budget across partial frames, a 4MiB frame bound and bounded queued event count.
The real Unix-stream trickle regression demonstrates deadline enforcement; it is
transport evidence, not simulated runtime identity or termination evidence.

## Actual proof and chronology

The table links retained real RED and subsequent GREEN runs. Their Rust test
commands use `cargo test --offline --manifest-path crates/sqlite-controller/Cargo.toml
[--features fault-injection] --test codex TEST_NAME -- --nocapture`; unit transport
uses the named `native_ws` test. Test feature/fixture configuration is retained in
the scripts and compiled test targets/logs. Fault hooks exist only in the explicit
`fault-injection` build. Every RPC receipt and process identity remains native;
no successful reply or terminal identity is manufactured.

| Behavior | Initial RED | Actual GREEN |
| --- | --- | --- |
| Authenticated native registry projection | raw/red-registry.log | raw/green-registry-projection.log |
| Explicit configured migration (diagnostic only) | raw/red-configured.log | raw/green-configured.log |
| Supervised pinned private-network native service | raw/red-profile.log, raw/red-native-identity.log | raw/green-profile.log, raw/green-native-identity.log |
| Native bridge incarnation fence | raw/red-native-connection-fence.log | raw/green-native-connection-fence.log |
| Real shared/separate controller-owned shell+patch routing | raw/red-routing.log | raw/green-routing-rpc-diagnosis.log |
| App restart registration/input gate and actual tool restoration | raw/red-native-recovery.log, raw/red-recovery-tool-effects.log | raw/green-recovery-tool-effects.log |
| Replaced native store refused | raw/red-native-store-identity.log | raw/green-native-store-identity.log |
| Actual native turn-start lost acknowledgement | raw/red-native-lost-ack.log | raw/green-native-lost-ack.log |
| Durable topology/config identity | raw/red-native-config-identity.log | raw/green-native-config-identity.log |
| Revoked input eligibility/retirement | raw/red-native-retirement.log | raw/green-native-retirement-fixture-repaired.log |
| Active turn cancellation distinct from executor stop | raw/red-native-active-cancel.log | raw/green-native-active-cancel.log |
| Absolute native wire deadline | raw/red-native-wire-deadline.log | raw/green-native-wire-deadline.log |
| Confirmed thread creation adopted after controller exit79 | raw/red-native-confirm-adoption.log | raw/green-native-confirm-adoption-same-incarnation.log |
| Configured restart cannot silently omit adapter | raw/red-native-missing-config.log | raw/green-native-missing-config.log |
| Typed scoped turn/activity/capability projection | raw/red-native-scoped-projection.log | raw/green-native-scoped-projection.log |
| Unknown actual start fences another turn | raw/red-native-unknown-start-fence.log | raw/green-native-unknown-start-fence.log |

`native_registry.py` exercises both topologies with actual native initialization,
code-mode-host `functions.exec`→shell and apply_patch, distinct PID/net namespace
observations, missing host/peer/DB/controlplane/auth paths and blocked host TCP.
It records complete nonsecret profile/config bytes and exact component identities.
Actual native history, RPC observations and marker effects are retained in full.
Controller/app/executor restart injections target only retained own process pidfds.

Disconnect coverage is honestly supplemental to already working remote-only
routing. `raw/red-native-disconnect.log` failed a fixture assumption that a
recovering native turn would become terminal in20s. The corrected observer sees
actual disconnected/recovering status with the retained remote environment and
no shell/patch local fallback; interrupt/rebind then performs actual remote
shell+patch. `raw/red-native-disconnect-observed.log` **passed despite its filename**;
no product change satisfied this case. Do not classify it as a product RED.

Other intermediate failures remain retained: initial profile transport/mount
mechanics, pending environment timing, too-short native RPC deadline, and a
fixture cleanup variable shadow. `green-native-recovery-first.log` passed its
initial gate but exposed actual read-only tool failures; strengthened real tool
assertions produced `red-recovery-tool-effects.log` before the policy fix.
The first confirmation-adoption retry revealed native 'no rollout found'; a
later retry exposed only a fixture counting autonomous resume rows as forbidden
client operations. Its corrected assertion counts only `codex_*` client ops.
Neither failed GREEN-prefixed run is represented as passing qualification.

Source provenance limits: retained source JSON/text inventories and tar archives
are included verbatim. Not every initial RED has a contemporaneous whole-crate
snapshot. Some archives were captured while a previously compiled fixture ran,
after independent later edits; those are contextual snapshots, not assertions
of exact compiled whole-crate RED bytes. Pre-run source inventories exist for
active cancellation, confirmation crash, missing configuration and disconnect
coverage; compare any archive against the inventory before treating it as exact.
No historical order or RED is reconstructed/fabricated. Final gates instead pin
a frozen complete source inventory, actual commands/exits and all retained bytes.

Late lead replacement/retired-authority assertions and process disposal inventory
strengthen the already green actor/revocation slice without additional product
behavior changes. The paired topology ruling and transient-own-executor bearer
scope were explicit parent decisions. Prior process-core handoff/repair evidence
is unchanged and remains authoritative for its own history.

## Gate, resources and downstream boundary

Raw logs are retained verbatim, including terminal blank lines; style checks
exclude these immutable raw bytes. Final commands/exits and source pins are in `raw/gate-commands.json`,
`raw/source-sha256.txt` and the retained final test/fmt/clippy/normal-build logs.
The initial full50-test gate passed before a whitespace-only launcher correction;
its log/archive remain unchanged. The second full gate passed50 tests on the final launcher
content pin:1 native wire unit +10 native integration +4 process +20 public +15
runtime. Native integration took346.40s with explicit `--test-threads=2`; test,
formatter, strict clippy, normal locked/offline build and diff check each exited0. README prose is the sole documentation difference from the first
pre-run source inventory; implementation hashes are separately checked.

`raw/gate-resources-final.json` retains86 actual whole-process pidfd disposal
receipts and read-only final observations of65 unique retained qualified
identities: zero matching live survivors. The initial gate resources remain
retained separately in `raw/gate-resources.json`. Fixture teardown can kill a still-owned
service namespace through its qualified retained pidfd; this is clearly separated
from product cancellation/slot proof. `raw/prior-fixture-resource-disposal.json`
checked120 earlier qualified own namespace identities: all were already absent,
so it signalled no process. Remaining resources are this worktree/branch, Cargo
cache/target and retained evidence. Private temporary roots, sockets and fixture
provider threads are disposed. No foreign/user process is inspected for cleanup
or signalled; there is no host PID-reuse forcing or host ptrace configuration change.

Reproduce from the delivered exact branch tip with installed pinned binaries and
catalog origin in `native_profile.py`/`native_registry.py`:

```sh
cargo test --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml --features fault-injection -- --nocapture --test-threads=2
cargo fmt --manifest-path crates/sqlite-controller/Cargo.toml --check
cargo clippy --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml --all-targets --features fault-injection -- -D warnings
cargo build --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml
sha256sum -c docs/efforts/sqlite-controller/evidence/02/registry/raw/source-sha256.txt
sha256sum -c docs/efforts/sqlite-controller/evidence/02/registry/evidence-sha256.txt
```

Linux process pidfds, actual namespace support, adjacent installed gate, pinned
Codex+companion/model catalog, read-only host `/usr`, and supervisor-restart unknown
capacity remain limits. Current stdio is null input with retained output; no PTY
or detach/reattach gateway is qualified. Native app bearer is global server
control, including dynamic tool responses, sibling reads and helper turns.
Raw UI connection is not scoped observation authority. Ticket05/06 must filter
reads/tool calls and refuse raw UI mutations/unrecorded turns before live use.
Parent-owned native reader's cross-thread live-session positive/negative controls
are additive protocol facts; they are not substituted for this controller's
namespace/network/authority proof. Private Git publication, terminal attachments,
credential gateway, actual question delivery and model turns remain downstream.
