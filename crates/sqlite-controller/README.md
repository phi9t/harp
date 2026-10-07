# Local SQLite controller

The retained disposable real-model demo is launched from the repository root:

```sh
python3 tools/agents/sqlite_baseline.py start
```

It builds into its own scratch target, creates a new `corenius-baseline-<run-id>`
tmux session, and prints its private run directory, attach command and status
command. The outer window prepares a real Kata project and immutable filesystem
intent, then pastes literal input into the scoped real Codex lead TUI. The lead
admits the workers and performs the existing exact-candidate verification,
acceptance and conditional fast-forward landing in a new disposable canonical
repository. The second phase retains two concurrent candidates, the stale second
landing, and a freshly verified integration Attempt. Worker windows show scoped
process/thread/progress/candidate state; the dashboard shows the controller view.

The current prerequisites are the pinned Codex0.159.3 and companion, the qualified
Kata0.24 binary and native catalog/config paths declared in the runner, working
unprivileged bubblewrap, cargo's offline dependencies, tmux, and an unexpired
current host ChatGPT login. A fixed host Unix Responses relay reads that login
without copying or refreshing it. Only sterile app namespaces receive the relay
socket; executor networking remains isolated. The relay retains request hashes,
byte counts, status and intervals, and does not retain headers or response bodies.

Use the printed `status` command to read the actual result. A successful launch
is not a completed trial. `result.json` appears only after all three tasks land;
`failure.json` retains the first setup/target failure. The run keeps its newly
created tmux session, services, candidate artifacts, proofs, dashboard snapshots
and process records for observation. Existing sessions and repositories are
preserved. A separate manual lead input can be sent with
`python3 tools/agents/sqlite_baseline.py input RUN_DIRECTORY --file UTF8_INPUT`.
Every invocation starts a fresh disposable project. This demonstration does not
qualify the permanent baseline, backup/restore, or broader failure recovery.

Standalone storage and deterministic supervised runtime baseline; no PostgreSQL
or Temporal dependency or service. Build and
run from this directory, or pass `--manifest-path crates/sqlite-controller/Cargo.toml`.

```sh
cargo test --locked --offline --features fault-injection
cargo build --locked --offline
mkdir -m 700 /tmp/corenius-private
# Set a retained random CORENIUS_OPERATOR_TOKEN through your secret handling.
./target/debug/corenius-sqlite-controller /tmp/corenius-private/state.db /tmp/corenius-private/api.sock
```

The database directory must already exist on persistent supported local storage
outside all worker mounts. Database, WAL, shared memory, socket and lifetime lock
are controller-owned. The lock file is never removed; OS ownership ends on exit.
Dangling database symlinks are refused before creating a target. Existing symlink
aliases resolve to the same canonical lock; database hard links are unsupported.
Do not run the daemon with the `fault-injection` feature in normal operation.
That explicit test build supports a pre-COMMIT admission barrier, not a runtime effect.

Send one newline-delimited JSON request per Unix socket connection. Socket reading
and response writing each have a one-second framing deadline. Controller effects
use their own bounded execution budget. Up to 16 connections run concurrently;
SQLite retains one transaction writer. An expired connection closes. Retry an unacknowledged mutation with its original key. Replies are
`{"ok": VALUE}` or `{"error": "DESCRIPTION"}`. The library exposes the same
`Controller::open(path, bootstrap_secret)` and `Controller::handle(JSON)` seam.
The operator principal is `operator`; the secret initializes it once and must
match on reopen. No token rotation or historical-state import is implicit.

```python
import json, os, socket
request = {
    "auth": {"principal": "operator", "token": os.environ["CORENIUS_OPERATOR_TOKEN"]},
    "project": "demo",
    "command": {
        "kind": "create_project", "expected_revision": 0,
        "idempotency_key": "project-demo",
        "args": {"name": "demo", "base": "0123456789abcdef0123456789abcdef01234567"},
    },
}
with socket.socket(socket.AF_UNIX) as s:
    s.connect("/tmp/corenius-private/api.sock")
    s.sendall((json.dumps(request) + "\n").encode())
    print(s.makefile().readline())
```

A command receipt contains `receipt`, `event_seq`, `revision`, and `result`.
Use the returned project revision for the next mutation. Retry a lost response
with the identical request and key; changing any argument or expected revision
with that key is rejected. A conflict uses a new key and current revision.
Authentication and current authority are checked before replay. Failed commands
do not consume keys or append events. Keys are scoped by project, principal and
authority epoch. JSON object field order does not affect identity.

Replace `command` with `"query":{"kind":"snapshot","args":{}}` to inspect
state. `events` accepts `after` (exclusive global sequence cursor) and `limit`
(default100, maximum1000). Advance to the last event actually returned; gaps
are normal across projects/scopes. Queries return only the authenticated scope.
Own attempt observations and operation transitions derive event task identity
from their retained attempt/operation, so lifecycle changes reconnect correctly.

Commands and required arguments:

| Command | Arguments | Authority |
| --- | --- | --- |
| `create_project` | `name`, `base` | operator |
| `create_task` | `task`, `work_ref:{deployment,project,item}`, `source_revision`, `brief`; optional checked `brief_digest` | operator/lead |
| `admit` | `task` | operator/lead |
| `replace_lead` | `{}` | operator |
| `pause_dispatch` | `paused` boolean | operator/lead |
| `abort_task` | `task` | operator/lead |
| `cancel_attempt` | `attempt` | operator/lead |
| `observe_attempt` | `attempt`, `state`, `observation` | operator |
| `transition_operation` | `operation`, `state`, `observation` | operator |
| `ask_question` | `task`, `text` | operator/lead/own attempt |
| `answer_question` | `question`, `text` | operator/lead |
| `record_decision` | `text` | operator/lead |
| `send_message` | `recipient` principal, `text` | scoped principal |
| `ack_message` | `message` | recipient only |
| `report_progress` | `task`, `text` | operator/lead/own attempt |

Project base must be a full 40- or 64-hex Git commit OID, checked again at admission.
Symbolic refs, abbreviations and malformed IDs are refused atomically. Object
existence/retention belongs to ticket04.

`admit` returns the attempt, slot, prepared start operation, pinned base/brief,
and attempt principal/token/epoch. `replace_lead` returns a fresh lead grant;
old lead grants fail immediately, while attempt authority stays independent.
Token-bearing receipts stay private to their authenticated issuer.
`dispatch_context` query (`attempt`) is operator-only and retrieves the retained
current unpaused attempt grant for the controller-owned adapter. `pending_operations`
and `storage_configuration` queries are operator-only. Snapshots and events never
expose grant tokens; attempts see their own tasks, questions, operations and
sent/received messages. Message and acknowledgement events use retained sender/
recipient identities, including acknowledgement replay to the original sender;
caller task metadata never widens a message audience. Decisions are shared within the project.

There are two slots across the entire controller. WorkRef is canonical per project, so aliases cannot double-admit the same work item.
Admission pins immutable brief
and source revision/digest and journals a start before acknowledgement. Dispatch
uses the supervised runtime described below. `prepared`, `applying`, and `unknown` operations survive
restart for reconciliation; restart does not blindly dispatch or release slots.
Cancellation records its own prepared operation, revokes the attempt grant, and
retains capacity. Task abortion changes only task outcome. Only privileged
`observe_attempt(state="exited")` releases a slot in the storage-only library
fixture. The socket service refuses externally supplied terminal observations
and operation transitions; runtime mode refuses all external attempt observations.
Only the controller-owned adapter translates validated supervisor evidence into
these storage mutations. `Controller::handle` without the runtime remains an
unqualified storage fixture, unsuitable as a product lifecycle endpoint.

The supported operation transitions are prepared → applying/failed/unknown,
applying → confirmed/failed/unknown, unknown → applying/confirmed/failed.
Confirmed/failed are terminal. Observed attempts may move to running, unknown,
or exited; exited is terminal. No operation outcome implies an attempt exited.
Runtime effects must happen between separate journal commands, outside SQLite
transactions. Future adapters must reconcile ambiguous starts before retrying.

Version1 is a fresh-project migration applied atomically; unknown schema versions
fail closed. Candidates, verification and acceptance have separate immutable
records under the qualified Git adapter below. No backup/restore procedure is
qualified. The optional Kata adapter is described below.
Do not copy a live DB without its WAL, overwrite it with an older backup,
or infer power-loss durability from process-kill tests. A supported restore path
must add explicit authority rotation and external-effect reconciliation.

The ticket02 runtime requires Linux with process pidfds, `/usr/bin/bwrap` with unprivileged namespace
support, and the standard `/usr` toolchain. Configure a dedicated supervisor
outside controller and terminal lifetimes. Its root, socket, secret and controller
storage must stay outside the `/usr` toolchain mount and worker mounts. Retain the
supervisor root on supported local persistent storage alongside the SQLite
authority; never delete or restore its execution records independently.

For example, create a private configuration with an operator-selected executable
and literal argument vector (no shell expansion):

```json
{"root":"/tmp/corenius-runtime/executions","profile":{"executable":"/usr/bin/sleep","args":["120"]}}
```

The root directory must already exist, use its canonical absolute path, and be
dedicated to this controller deployment. Set distinct retained secrets
`CORENIUS_OPERATOR_TOKEN` and `CORENIUS_SUPERVISOR_TOKEN` through your secret
handling. Start these as separate services:

```sh
./target/debug/corenius-attempt-supervisor /tmp/corenius-runtime/supervisor.json /tmp/corenius-runtime/supervisor.sock
CORENIUS_SUPERVISOR_SOCKET=/tmp/corenius-runtime/supervisor.sock \
  ./target/debug/corenius-sqlite-controller /tmp/corenius-private/state.db /tmp/corenius-private/api.sock
```

The configured daemon automatically drains and reconciles all projects on startup
and every 250ms in its owned reconciler. Native UI operations share its lifecycle
gate, while controller queries and scoped tools can answer native calls. Normal clients issue admission
and cancellation through the team API; the lead and outer observers never
dispatch through the supervisor. This operator-only request additionally allows
an explicit reconciliation step for diagnosis:

```json
{"auth":{"principal":"operator","token":"YOUR_RETAINED_OPERATOR_SECRET"},"project":"demo","runtime":{"kind":"reconcile"}}
```

The daemon observes exits and cancellation completion automatically. Admission
and cancellation receipts record durable intent before any process effects.
Cancellation revokes worker authority immediately and keeps the slot until
identity-bound termination or a durable never-started tombstone is observed.
Pausing dispatch suppresses pending starts while permitting observation and
cancellation. Reconciliation accepts no caller-selected identity, lifecycle
state, principal, executable, profile or host path. The adapter uses short journal
transactions separated from all supervisor transport and process effects.

One serialized supervisor owns each operation-keyed effect. It persists a random
execution handle, original supervisor incarnation and kernel boot ID before
dispatch. Live records additionally retain monitor PID/start ticks and actual
namespace-init PID/start ticks/PID namespace inode, validated through the kernel.
The supervisor opens a provisional kernel pidfd, validates its process target,
namespace PID1 identity, start identity and parent relationship to the owned
monitor, then confirms process liveness before retaining it as cancellation
authority. Qualification failures drop the provisional descriptor without
signalling it.
The adjacent installed `corenius-attempt-gate` binary is mounted read-only and
blocks workload exec on a private stdin gate until identity is validated and
durably recorded, then restores null stdin before exec. Short-lived profiles
cannot race identity sampling. Setup errors terminate owned children and retain
unknown capacity when whole-process termination cannot be established.
The controller retains and checks that immutable identity on adoption. Lost start
replies are resolved through the original operation ID before dispatch; an
acknowledged execution missing from the supervisor is unknown and is never
restarted. The supervisor's lifetime lock and durable cancellation tombstones
prevent duplicate starts and late-start cancellation races.

Each execution runs in Insula's bubblewrap boundary with private workspace, home
and scratch, cleared environment, isolated PID/network namespaces, a private
`/proc`, and read-only `/usr`. Host home, controller database/socket, supervisor
records/socket, SSH agent, canonical Git metadata, credentials and other attempts
are absent. The worker is namespace PID1, so Linux terminates its descendants
when its whole process exits, including descendants that create new sessions. Cancellation
signals the retained namespace-init pidfd and terminates the owned monitor;
capacity release requires process-level pidfd readiness. A zombie leader with
live threads is still running for this proof, and parent-death signalling is not
an authority substitute. Logs remain in the private attempt
root, outside worker writable mounts. Roots and preparer-returned mount paths
refuse aliases and symlinks. `WorkspacePreparer`/`serve_with_workspace` expose a
trusted controller/operator hook receiving the pinned base and brief identity
for ticket04's private Git backend; the default creates empty fixture workspaces.

Supervisor restart deliberately leaves all retained nonterminal effects unknown:
it cannot reconstruct `Child` ownership from a PID. Controller restart with the
original supervisor alive adopts the same handles. Supervisor unavailability,
ambiguous setup or identity mismatch retain occupied slots; there is no unsafe
operator text override or automatic cleanup/recovery from supervisor loss.
Services are trusted local processes; workers cannot reach their private sockets
or secrets. Raw supervisor protocol access is a controller-owned internal seam.

The simple command profile has null stdin and retained stdout/stderr. The native
profile below additionally qualifies model-free Codex protocol and scoped fixture
provider connectivity. Both provide no PTY, attempt tool gateway or live-runtime qualification.
The `/usr` base is a read-only host toolchain, not a hermetic dependency image.
Private Git workspaces and candidate evidence are described below. Gateways, PTY
attachment and the dedicated live Codex trial remain later tickets. Terminal sessions may observe
these executions, but must not become their owner.

Run the reproducible standalone gate (real bwrap tests fail if namespaces are
unavailable, rather than silently skipping isolation):

```sh
sh tools/agents/checks.d/70-sqlite-controller.sh
cargo fmt --manifest-path crates/sqlite-controller/Cargo.toml --check
cargo clippy --manifest-path crates/sqlite-controller/Cargo.toml --locked --offline --all-targets --features fault-injection -- -D warnings
```


## Controller-owned Codex registry

Opt in with `CORENIUS_CODEX_CONFIG` pointing to a private, trusted JSON file.
The explicit `schema_migration: "1-to-2"` adds registry tables without importing
historical state. Configuration is retained exactly and changed topology/endpoints/
auth references are refused on restart; a nonempty configured registry also refuses
a restart without this configuration. Version1 remains the simple command-profile
baseline; an empty `servers` array is a schema diagnostic, not a qualified runtime.

```json
{
  "schema_migration":"1-to-2",
  "topology":"shared",
  "transport_root":"/tmp/corenius-native/t-app",
  "servers":[{
    "id":"native",
    "supervisor_socket":"/tmp/corenius-native/controlplanes.sock",
    "auth_ref":"CONTROLPLANE_SECRET",
    "transport_root":"/tmp/corenius-native/t-app",
    "worker_supervisor_socket":"/tmp/corenius-native/workers.sock",
    "worker_auth_ref":"CORENIUS_SUPERVISOR_TOKEN",
    "worker_transport_root":"/tmp/corenius-native/t-exec",
    "schema_digest":"e587a3d5ffab12e528589e14d385b04d1192b9ae6914ce008cd3ea507e5aa455"
  }]
}
```

Run separate independently owned supervisors for worker executors and sterile
native app-server controlplanes. The worker supervisor must be the same endpoint
configured by `CORENIUS_SUPERVISOR_SOCKET`. Each supervisor's trusted `profile`
uses executable `/usr/bin/python3`, args `["/run/codex/launcher.py"]`, and `codex`
with `kind` (`app` or `exec`), `transport_root`, and five `{path,sha256}` pins:
`binary`, `companion`, `catalog`, `config`, `launcher`. App profiles also supply a
fixed `provider_socket`, fixed `executor_root`, and `own_executor_only` (`false`
for shared, `true` for separate). See the retained exact fixture configuration
in [registry evidence](../../docs/efforts/sqlite-controller/evidence/02/registry/handoff.md).
The pinned binary is Codex0.159.3 and its adjacent code-mode-host; the launcher is
[support/codex_insula.py](support/codex_insula.py). Asset hashes are validated,
then read-only sealed descriptors are mounted into each namespace. This has no
caller-selected executable, mount, model or raw runtime endpoint.

Shared topology has one sterile app-server/thread store and distinct private
executors. Separate topology pairs each private executor with a separate sterile
app-server/store/companion namespace. Tools cannot inspect either controlplane's
processes or sockets. Private network bridges reach only fixed provider and own
executor endpoints. Plaintext app capabilities stay outside tools; a transient
bearer inside its own native executor is limited to that already isolated tool
realm. It does not grant shared app-server, peer, host, upstream credential or
controller submission authority. The host read-only `/usr` toolchain remains a
nonhermetic dependency.

The daemon owns admission dispatch, native binding, reconciliation and cleanup;
normal clients use the existing team admission/cancellation API. Operator-only
`codex_registry` returns protected auth references and qualified server/namespace/
execution/store/workspace identities, never plaintext native capabilities. Lead
bindings use role/authority epoch and consume no worker slot. Worker bindings use
the admitted immutable brief, attempt scope epoch and owned execution handle.

`codex_turn` takes `attempt` (or `role:"lead"`) and `text`; `codex_steer` and
`codex_interrupt` also require `expected_turn`. Optional `expected_binding`
checks retained `thread_id`, `server_store`, `server_incarnation`, `environment_id`.
No command accepts caller routing identities, history, filesystem paths,
environments, workspaces or terminal observations. Current actor and issuer
permissions are rechecked immediately before RPC. Starts with unknown/applying
acknowledgement fence further turns until typed native history reconciles the
actual durable user-message client ID and input; native request IDs are not
idempotency keys. Native completion never proves task success or executor death.

Every native mutation has durable intent before RPC, outside SQLite transactions.
Controller restart reuses receipt identities and ID-only native subscription
resume. Native app restart restores registration and controller permission policy,
then keeps input eligibility blocked until an explicitly remote turn selects the
qualified executor and typed thread observations validate it. The controlplane
cwd is `/workspace` in its sterile namespace; selected environment cwd/roots are
`/workspace` in the private executor namespace. These are separate identities.
A newly created native thread has no resumable rollout until its first turn:
confirmed creation can be adopted only in the same surviving app incarnation.
A missing creation acknowledgement, changed-incarnation unmaterialized thread,
identity mismatch or unproved effect remains unknown without blind recreation.
Supervisor restart ambiguity retains the process core's capacity restriction.

Cancellation withdraws input authority first, journals interruption of the
retained native turn and records typed terminal receipt or explicit uncertainty.
Whole executor termination and slot release remain a separate pidfd observation.
Native WebSocket RPC uses an eight-second absolute send/receive deadline, a 4MiB
frame limit and a bounded event count; oversized histories fail closed.

Native fixtures are `MODEL_FREE_ACTUAL_PROTOCOL`: deterministic local Responses
SSE drives the actual pinned app-server, code-mode-host, shell and apply_patch
inside qualified namespaces. They make no inference/model calls. Reproduce with
`cargo test --locked --offline --features fault-injection -- --nocapture --test-threads=2`.
The fixture needs the pinned local binaries and model-catalog origin recorded in
the evidence. The `fault-injection` build alone enables native acknowledgement
loss, adapter hold and post-confirmation crash hooks; production builds omit them.

A native raw endpoint/bearer is server-wide authority, including dynamic tool
responses and helper turns. It is controller-only. No raw TUI or worker gateway is
qualified here; ticket05/06 must filter scoped observations and deny unrecorded
mutations/tool replies before attachment or live use. PTY ownership, provider
credential gateways, private Git publication and live model qualification remain
explicit downstream gates.

Confirmed initial native turn receipts are adopted before classifying a thread as
empty; the retained turn and typed current-turn IDs are saved together. Empty
thread recovery scans real loaded-thread cursors under one eight-second deadline,
with explicit 10,000-ID/100-page resource bounds. Malformed or repeated cursors
and resource exhaustion retain uncertainty; an exhausted valid scan refuses an
absent identity.

Revocation retains the actual native turn stop disposition before withdrawing
thread observation. Qualified native unsubscribe is a connection-scoped cleanup
operation journaled before RPC; lost acknowledgements reconcile using the pinned
idempotent unsubscribe statuses. Unknown store/incarnation qualification withdraws
local observation while retaining native cleanup uncertainty. The WebSocket
receive allowlist contains at most 1,024 current thread identities, preserves a
new thread's notifications only after its actual creation identity is returned,
and never accumulates an unbounded retired-thread set. The event queue remains
bounded at 1,024 frames; active peer frames are routed once to their exact thread.

## Pinned Kata and participant protocol

Opt in using `CORENIUS_KATA_CONFIG` with a private regular JSON file outside worker
mounts. `schema_migration: "kata-v1"` explicitly installs the additive Kata and
participant tables in the version1/2 database. This imports no tasks or old
backend state. Reopening durable Kata bindings requires the exact configuration;
changing a deployment, project, registered source root or actor fails closed.
The extension retains global schema version1/2 for the qualified runtime
interfaces. Only this exact qualified controller build/configuration is supported;
an older binary is not fenced by a new global version. No downgrade is authorized
or promised. The explicit extension imports no historical source authority.
Only the trusted service bootstrap reads the credential file. No worker response
contains that credential, its filename or an unrestricted Kata endpoint.

```json
{
  "schema_migration":"kata-v1",
  "controller_id":"my-controller",
  "actor":"corenius-controller",
  "api_schema_version":"0.24.0",
  "origin_instance_uid":"PIN_ACTUAL_INSTANCE_UID",
  "origin":"http://127.0.0.1:45678",
  "token_file":"/private/controller/kata.token",
  "project_id":2,
  "project_uid":"PIN_ACTUAL_PROJECT_UID",
  "repositories":{"intent-repo":"/canonical/registered/intent/repository"}
}
```

The supported contract is Kata commit
`b70a4306464e319701df30a3b92860289613b15a`, API0.24.0. Qualification checks
health, current `/instance` identity and authenticated actor, project UID, and
item creation origin through supported events before each source read/effect.
It never opens Kata's database. Use a controller-owned identity token. Native
Kata writable subtree tokens also allow claim/close/edit and cannot enforce the
Corenius worker boundary. Installed Kata0.14.3 is incompatible.

All mutations use the normal authenticated `command`, expected project revision
and idempotency key. Configured Kata mode refuses caller-authored `create_task`;
use `create_kata_task` with `task`, immutable item UID,
`expected_source_revision` (integer), and `source_files`:

```json
{"task":"task-a","item":"PIN_ITEM_UID","expected_source_revision":1,
 "source_files":{"storage":"intent-repo","mode":"git",
 "commit":"FULL_COMMIT_OID","paths":["spec.md","tasks/task-a.md"]}}
```

The operator selects the complete set of files necessary to interpret intent,
including referenced files. Relative paths are explicit and unique. Git mode
reads exact commit objects, independent of worktree dirt. For dirty/untracked
input use `mode:"snapshot"` without `commit`: the controller captures actual
bytes, exact HEAD provenance, dirty disposition, each SHA256 and an aggregate
snapshot digest. Snapshots use descriptor-relative traversal and read the same
opened regular file they validate, refusing symlinks even during path replacement.
All selections refuse traversal and unknown
storage IDs. Git selections retain exact blob bytes; a symlink target is not
dereferenced and any referenced target must be explicitly included.
Immutable version records retain the original bytes, including after filesystem
changes; the brief names their immutable version/digests. The live pathname is
not execution authority. A file-selection change requires a new admitted
contract workflow; this adapter does not silently revise any admitted brief.

`admit` rereads the qualified item and registered source selection. The actual WorkRef is globally unique across local-project aliases. Unlinked
storage-fixture tasks keep their history but cannot gain Kata admission authority.
A changed or closed task refuses assignment and durably retains the observed amendment.
`refresh_kata_source` (`task`) and autonomous service reconciliation also retain
current versions and proposals. Admission's original source revision and brief
never change. Refused admission can append a source-observation event without a
command receipt: fetch the current revision before another command. Successful
idempotent replay needs no HTTP and works offline.

`sync_kata_task` (`task`) explicitly queues publication. Admission, observed
attempt state, progress, questions/answers and cancellation also queue it in the
same transaction as their controller event. Every publication has separate
metadata and visible-comment effects, stable operation IDs and immutable
controller event/revision payloads. SQLite remains execution authority. Kata's
native open/closed status is independent: external closure proposes a source
amendment and never accepts a candidate, lands code or proves an execution stopped.

| Controller condition | Kata execution summary |
| --- | --- |
| created | unassigned |
| active with starting attempt | assigned |
| active with observed running attempt | running |
| unanswered question, unknown attempt, requested cancel, repair/integration required | blocked |
| submitted / accepted / landed / aborted | same named disposition |

Publication uses supported `corenius.execution` metadata guarded by the current
`If-Match: "rev-N"` plus the previous acknowledged JSON value (or absence), and
human-readable supported comments. The summaries distinguish reported progress
from observed attempt state. They include admitted version and controller event
and revision. Owned metadata revision changes update current/acknowledged Kata
revisions without generating source amendments; human intent, dependency, status
and other metadata changes remain proposals. External execution-summary drift
withholds publication and stays visible.

Reconciliation reads exact effects before retry. Metadata uses guarded patch and
readback; comments use `Idempotency-Key` and unique body/author readback. Lost
acknowledgements remain unknown until proof is retained. Server comment keys last
seven days: an absent ambiguous comment older than that is withheld for explicit
reconciliation, never blindly repeated. Operation histories retain earlier
failures even after confirmation. Publication remains ordered within each task;
an unresolved effect withholds that task's later effects while unrelated tasks
continue. Visible progress summaries use a bounded excerpt while the full
accepted report remains in the controller ledger. Successful unchanged-source
observations clear current availability uncertainty, preserve historical failures,
and independently retain actual execution-projection drift.
HTTP runs outside SQLite locks/transactions. The configured service reconciles
Kata on a separate controller-owned thread so slow upstream observations leave
the socket loop available for control requests; an operator can also send
`{"auth":AUTH,"project":"PROJECT","kata":{"kind":"reconcile"}}`.

`participant_context` is a scoped query returning admitted/current/proposed
sources, assignments, decisions, questions/answers, messages/acknowledgements,
followup proposals, pending effects/history and acknowledged Kata revisions.
A replacement lead reconstructs context without prior conversation. Native
binding/operation projections preserve pending cleanup uncertainty; this ticket
does not qualify native retirement or replacement-lead runtime completion.
Workers see their own assignment and linked-task intent projections, report
progress, ask questions, route/acknowledge messages and `propose_followup`
(`task`,`text`). Proposals do not create, assign or close work. Authenticated
sender identity is authoritative; caller-supplied sender fields are refused.
Workers cannot claim/close, change dependencies, issue grants or reconcile Kata.
Candidate execution/submission remains ticket04's separate contract.

The retained public qualification runs the exact real daemon with unique private
KATA_HOME/project/input repository, loopback ports and fixture-only credentials:

```sh
cargo test --locked --offline --features fault-injection --test kata -- --nocapture
```

The deterministic fixtures are gated by `fault-injection`; default builds/tests
do not run fixtures that rely on manual reconciliation or declared test barriers.
The fixture prints its actual retained path and records it in `qualification.json`.
It retains each fixture under the validated temporary root (`$TMPDIR/k3-*`), including real
Kata/controller databases, source Git objects, original source bytes, sanitized
public requests/responses, process identities/exits and failed runs. It injects
dropped acknowledgements with a forwarding proxy that only forwards genuine
responses. The alternative deployment control uses its own private bootstrap
credential to isolate actual instance-UID refusal. This is deterministic Kata
and participant proof; it makes no live-model or native cleanup qualification claim.


## Private Git candidates and protected-ref landing

Set `CORENIUS_GIT_CONFIG` to the same trusted, immutable JSON policy file for the
controller and executor supervisor. `Controller::configure_git(path)` installs it
for embedded callers. Policy fields are `project`, `root` (private controller
stores), `attempts_root`, `canonical`, `protected_ref`, pinned `git`, `python`,
`bubblewrap` tools (`path` and SHA-256), and `suites`. Each suite has `id`, `task`,
optional exact `base`, pinned `script`, and fixed `args`; a null base supplies the
fixed combined contract after integration. Missing or ambiguous selectors refuse
verification. Tool/suite byte changes and durable policy changes are refused.
The canonical repository is trusted operator input, outside worker mounts.

`TrustedWorkspacePreparer` provisions a new complete repository before the actual
supervisor starts a simple or native Codex executor. It seeds full Git objects
through a bundle, uses no linked worktree, hardlinked object or alternate store,
and pins the admitted base/brief/authority in an owner record outside worker
mounts. Existing attempt roots are refused. `GitBackend` exposes typed
`create_workspace`, `capture_candidate`, `materialize_candidate`, and
`cleanup_manifest` and `retire_retained` operations through this Git workspace interface; jj remains
unqualified.

The public commands are `capture_candidate {report}` (current attempt only),
`verify_candidate {candidate}`, `accept_candidate {candidate, verification}`,
`land_candidate {candidate, expected_trunk}`, `compare_candidates {candidate}`,
`retire_workspace {attempt, retire_refs}`, and `reconcile_git {}`. Queries are
`workspace_inventory {}` and `candidate_evidence {candidate}`. Candidate reports
are evidence content, and workers cannot choose suite commands or create proof,
acceptance or ref effects. All ordinary auth/revision/key fields remain required.
Git effect intents consume their original idempotency key before execution, so
an unknown response must be retried with identical arguments and original key.

Capture runs fixed Git commands inside a separate OS boundary. Untrusted worker
Git config, filters and hooks never run on the host. Bundle scratch is private
tmpfs; a retained no-follow regular descriptor validates bundle identity, link
count and a 16MiB bound before emitting a bounded bundle stream. Host stdout and
stderr reads are bounded at 24MiB and 1MiB, with a 30-second owned-process budget.
Complete commit/tree/base, bundle, manifest, report and their content digests are
retained and synced before acknowledging candidate submission. Historical
candidates, reports, acceptance and effect observations are immutable.

Verification independently materializes the retained commit in another complete
repository and executes the policy's pinned suite in an OS boundary with no
controller paths or network. Its report binds the candidate/base/brief, suite,
environment, logs and actual process identity. Acceptance is a separate authorized
command requiring the current candidate's latest passing required proof. Fresh
verification requires a current submitted task: accepted, landed or withdrawn
state refuses before creating a new journal operation. An identical prior
verification key still replays its retained proof. A later attempt or candidate
supersedes authority without deleting historical evidence.

Landing serializes the physical protected-ref effect, journals before effect,
requires fast-forward ancestry and compares the actual ref to `expected_trunk`
with conditional `update-ref`. Lost acknowledgements recover the same operation;
unexplained lineage or divergence stays unknown and blocks dispatch. A stale
candidate needs a new private attempt on the latest base and fresh combined
verification and acceptance. Shared branches are never rebased or deleted.

Inventory reports repository kind/path, attempt/base/current candidate, refs,
dirty/untracked state, actual process observation, retention, cleanup and drift.
Cleanup requires retained candidate/archive bytes, positively observed stopped
execution, exact workspace identities and explicit sole owned disposable refs.
Dirty, untracked, live, unknown or shared repositories are refused. A crash before
or after removal retries the same identity-fenced cleanup operation; missing
paths alone do not establish cleanup.

Git pending-journal recovery is qualified for the exact tested build and policy.
In-place upgrade or import of pending journals from rejected initial ticket04
(`2b1887b`) is unsupported. Missing admission/root provenance is preserved and
refused: landing stays unknown and blocks conflicting dispatch; missing-root
cleanup can return an error and record failure. Retain old databases, objects,
artifacts and process records. Changes after baseline acceptance must preserve
supported recovery or establish an explicit migration contract.

Ticket04 qualifies native first cleanup after an app restart only when no
unsubscribe operation exists. A fresh current terminal observation can qualify
that first operation while preserving the original terminal receipt. Existing
pending operations retain their original logical identity, connection, store and
incarnation fences. Mechanism qualification does not change any resource's
observed live, unknown or stopped disposition.

The control center and lead tools use the same authenticated controller socket.
Start `corenius-sqlite-controller --dashboard SOCKET PROJECT --once` for a current
view, or omit `--once` for an interactive terminal view. Set
`CORENIUS_CLIENT_PRINCIPAL` and `CORENIUS_CLIENT_TOKEN` to the retained operator or
lead grant. The dashboard shows mission, two worker slots, observed state,
reported progress, immutable source revisions and filesystem drift, questions,
verification, acceptance, landing queue, uncertain effects and monitoring.
Refresh reads durable events and state; reconnect can replay events after a saved
cursor through the public `events` query. A snapshot and its filesystem inventory
can have different observation revisions; the view reports that difference.

Enter a saved command object containing `kind`, `expected_revision`,
`idempotency_key` and `args`, or submit the same JSON file with
`--control SOCKET PROJECT SAVED_COMMAND_JSON`. Keep all fields unchanged after a
lost response. Controls include `pause_dispatch` (paused true/false),
`cancel_attempt`, `answer_question`, `revise_brief`/`approve_brief`,
`request_repair`, the existing exact-candidate verification/acceptance/landing
commands, and operator-only `replace_lead`. Each returns a durable receipt.
Cancellation requested and process stop observed are separate facts. Revised
briefs are proposals until admitted; a running attempt keeps its original intent.

`--mcp SOCKET PROJECT` provides JSON-line MCP `corenius_query` and
`corenius_command` tools. The fixed process credentials supply identity; model
arguments cannot select a principal, database, endpoint or runtime environment.
Worker tools are scoped to their admitted task. An observer grant can read bounded
monitoring records and retained inputs and publish typed advisory assessments;
it cannot execute a correction or runtime command.

For native Codex 0.159.3 attachment, run
`--native-ui SOCKET PROJECT 127.0.0.1:PORT` with fixed participant credentials and
a distinct private `CORENIUS_UI_TOKEN`. Attach the pinned TUI using `--remote`
and `--remote-auth-token-env`. The gateway supplies only the bound current thread,
validates store/incarnation/environment, journals literal human input and refuses
helper creation, sibling threads and raw tool responses. Its process and TUI are
replaceable attachments; the controller and separately supervised lead retain
lifetime authority outside the two worker slots. The upstream native bearer is
server-wide and stays private to the controller.

Native typed visible worker items are ingested incrementally under a retained
source identity. Reasoning and unfinished tools are excluded; credentials are
redacted. Complete allowed inputs are retained privately with original-source,
redacted-input and policy metadata, and authenticated `monitor_input` retrieval
checks bytes and digest outside the writer. Missing/changed/oversized inputs
surface as gaps. History starts uncovered; history and live review frontiers are
distinct. `seal_monitor_batch` retains an immutable bounded input and intent
snapshot; `publish_monitor_assessment` validates an explicit disposition for each
item. Same-version replay returns the existing assessment, while version reruns
append assessments without advancing an already reviewed cursor or repeating
credits/findings/corrections. Verified credit requires project-bound retained
passing verification bytes. Hypotheses and classifications remain advisory.

Findings need separate authenticated adjudication before `authorize_correction`.
Delivery records requested/prepared/presented, and only the addressed current
actor can acknowledge through its tool grant; transcript text never acknowledges.
Resolution cites a retained same-target assessment. Lead questions similarly
retain persisted/prepared/submitted/explicit-ack/resolved history, including
early acknowledgement, lost reply and refused steer reconciliation. No analyzer
schedules workers or runs a model inside the SQLite writer. Ticket06 owns the
recurring analyzer and raw-export corpus qualification. Ticket05 does not promise
migration of journals from rejected earlier releases or intermediate trial builds.

Question delivery batches are byte bounded as well as item bounded. Large texts
stay exact in SQLite and arrive as digest/byte/ID references; `corenius_query`
`question_input` with `args.question` retrieves the full text. Later questions can
progress without repeatedly rolling back an oversized first batch. Native UI
interrupt uses the pinned `threadId`/`turnId` request and a turn-derived identity;
exact input replay uses its saved command across runtime state transitions.
Assessment publication checks every sealed native artifact before reviewing its
coverage and binds those checked immutable identities inside the commit. Input
retention writes private stages, syncs them, then publishes without replacing an
existing final digest. Unreferenced crash stages never become reviewed inputs.

## Bounded control-center history reads

`control_center`, `lead_context` and `snapshot` return bounded read pages with
`pagination`. Each collection contains at most 20 records and a 32KiB serialized
page. `control_records` accepts `collection`, `after`, `limit`, and a binding for
native history; follow the returned `page.continuation`. Control-center cursors
are passed as `args.cursors[collection]`. `event_page` returns the same explicit
continuation for reconnects. A page is an observation, never reviewed coverage.

Large text uses exact SHA256/UTF8-byte references. `question_input` retrieves the
full authorized question. `control_input` follows its reference with an optional
UTF8 byte `offset`, returning at most 65536 bytes and `next_offset`. Changed
references and offsets inside a character are refused. Full retained native
public messages/completed tool inputs use `runtime_public_input` references;
private reasoning, unfinished calls and credential fields are excluded.

Native bindings expose public-history references and runtime-operation pages.
Worker public history retains its original monitor stream identity and credit
rules. Lead history has a separate append-only read ledger: a current lead or
operator may inspect an earlier lead binding's retained public inputs, while
worker, observer and revoked lead grants are refused. Missing prior typed
retention is an explicit coverage gap; no historical backfill is promised.
The current native attachment still validates its own thread/store/incarnation.
A recovered start response presents matched native turn ID/status/error with
empty items as a history-derived presentation. Complete allowed items are
retrievable through the separate typed history reference; these empty items do
not describe the lost acknowledgement.

Configured Git inventory observes at most 20 selected owned workspaces outside
the writer lock. Workspace and effect-history pages expose coverage and
`workspace_inventory` continuations, including large immutable JSON input
references. Unknown inspection and retained artifacts remain distinct from a
clean result. Public pages are separate from the operator-only `runtime_context`,
which selects the two occupied slots and their unmodified start/cancel journals;
missing, duplicate or foreign journals refuse runtime reconciliation.
