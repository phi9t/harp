# One root Insula per Corenius instance

Status: proposed architectural extension for the owner-requested retained baseline; not an implemented deployment or a revision to an already admitted worker contract.
Route: gated architectural design. Apply the repository Matt–Superpowers crosswalk; unavailable Matt design owners are represented by its combined criteria. Independent critique and bounded host feasibility evidence accompany this design. Product implementation and readiness require the qualification below.

Source read: Corenius main `a21316f` (`src/insula.rs`, `tools/insula/capsule.py`); baseline documentation `6b16580f6bf43505cdb4423fa2ceda39767e048a`; standalone Insula `9e9b2a81e56eb682e534dec019f753c54c10bdb2` (README, CONTEXT, bwrap/plan code). Observations describe those sources, not all future versions. Recommendations below are design proposals.

## Goal and approach

Run each complete local engineering team inside one independently owned root Insula: controller, SQLite, Kata adapter/local fixture, runtime gateway, lead, two workers, verifier, dashboard, monitoring and tmux. Narrower child Insulas constrain each reasoning process and its tools. Multiple instances may use identical internal paths and local ports without sharing mutable state or control authority. Restarting a controller or closing a view must not implicitly terminate the root or surviving workers.

Insula vocabulary: the immutable **rootfs image** contains the software; a **capsule** combines it with a declared runtime mount/environment/command plan. “Root Insula” here means the root capsule for one team instance, not host root privileges. Child capsules share only explicitly admitted inputs. Insula's current reference implementation does not claim a hostile-code sandbox; hardening and negative qualification are additional Corenius work.

Alternatives: private directories alone prevent some name collisions but leave processes, loopback and inherited control resources shared. Independent flat capsules avoid nested-launch requirements but need an explicit external grouping and networking arrangement. Recommend the nested team root with a small host supervisor; record flat capsules as a distinct alternative if nesting is unavailable, never silently call host execution isolated.

```text
Host: outer qualification harness + instance supervisor
  └─ root capsule: instance I / root incarnation R
       ├─ independent root init / execution supervisor
       ├─ controller + SQLite + Kata adapter/local fixture
       ├─ canonical Git + artifacts + monitor ledger
       ├─ runtime gateway + scoped participant bridges
       ├─ dashboard + operator tmux server
       ├─ lead capsule: Codex app-server, own state, scoped lead tools
       ├─ worker A capsule: own app-server/state/private Git/Attempt
       ├─ worker B capsule: own app-server/state/private Git/Attempt
       └─ verifier capsule: exact Candidate, restricted evidence channel
```

Separate app-server per role/Attempt is the first qualified configuration, all beneath the same root. A shared app-server remains possible only after demonstrating per-thread tool execution isolation and capability confinement; distinct cwd or thread ID is not a filesystem/process isolation mechanism. Replacement retains binding provenance and continuity while changing RuntimeIncarnation or Attempt where required.

## Instance identity, storage and ownership

One host-private directory, exclusively owned and mode 0700, per immutable instance ID. Admission validates directory identity through safe handles, rejects aliases/symlinks and obtains an OS lifetime lock tied to that directory; a PID file alone is not a lock. A copied instance must be explicitly imported under a new identity with authority rotation and survivor reconciliation. Two supervisors must not acquire the same state.

| Projection | Owner and lifetime | Who can access it |
| --- | --- | --- |
| `/state/controller/` | SQLite database directory including WAL/SHM, controller config, operation journal; persistent | Controller and trusted recovery/backup path only |
| `/state/git/` | Instance canonical repository and landing records; persistent | Trusted integration path only |
| `/state/artifacts/` | Immutable manifests, evidence, retained checkpoints; persistent | Writes through broker; selected read-only artifacts projected to participants |
| `/state/roles/lead/` | Private lead runtime store and workspace | Lead runtime and scoped supervisor |
| `/state/attempts/<id>/` | Own Git metadata, runtime state, home, scratch and retained logs | Only that Attempt plus trusted supervisor |
| `/state/kata/` | Dedicated local Kata deployment state, if one is used | Adapter/local Kata process only |
| `/state/monitor/` | Transcript binding, cursor/assessment ledger, context policy | Instance monitor and trusted controller |
| `/run/corenius/`, `/tmp/` | Private sockets, locks and temporary files; recreated for each root incarnation | Individually scoped projections; never broad worker mount |

SQLite is embedded in the controller, not an extra database daemon. Bind its entire persistent directory on a qualified local filesystem, not just the main database file and not a shared network filesystem. Keep the baseline's WAL/FULL and backup/recovery contract. Agents and dashboard query controller projections rather than opening the database themselves. [SQLite WAL documentation](https://www.sqlite.org/wal.html).

Set `HOME`, `CODEX_HOME`, XDG state/config/cache/runtime directories, tmp/cache/build locations, Git config and Kata deployment/project explicitly per role/instance. These are a freshly constructed allowlist, not inherited host environment. Codex stores local state under `CODEX_HOME`; isolate it rather than binding host `.codex`. [Official Codex configuration](https://learn.chatgpt.com/docs/config-file/config-advanced).

Private Git repositories include their own `.git` and, when qualified, `.jj`; no shared writable worktree metadata, alternate objects or host canonical mount. A root capsule owns its canonical repository. If several instances ultimately target the same remote protected branch, filesystem separation does not serialize remote landing: require one shared landing authority or a qualified conditional remote update/reconciliation protocol. The initial trial uses separate destinations; no claim of cross-instance shared-mainline safety follows from it.

Bound writable disk capacity and inode consumption per instance, with separate Attempt/scratch/log allocations and controller persistence headroom. Use qualified filesystem quotas or equivalent storage containment; CPU/memory/process limits cannot prevent one instance exhausting another's storage. Protect retained Candidates/evidence from indiscriminate log cleanup. If the host cannot enforce storage containment, advertise namespace/state isolation with that explicit limit, not general noninterference. Qualification fills A's allocation while B continues committing/checkpointing its SQLite database and retaining artifacts.

## Process, filesystem, network and credential rules

Root admission creates private mount, PID, IPC, UTS, user and network namespaces with a fresh root filesystem. A host-owned cgroup contains the full instance and enforces configured process/memory/CPU ceilings; a cgroup namespace alone provides neither resource accounting nor limits. Do not bind the host's general cgroup control hierarchy inside it. Namespaces still share the host kernel; trusted same-UID host processes remain outside this isolation claim unless separate OS identities or stronger mediation are deployed.

Use separate trusted and Attempt cgroup leaves beneath the instance group. A trusted host mediation channel places an execution into its assigned leaf before untrusted code can fork, without exposing cgroup control to the worker. Cancellation covers daemonized descendants and waits for observed emptiness before workspace reuse/cleanup. Qualify delegated cgroup v2 and subtree termination on the target host; PID namespace separation alone is insufficient evidence. [Kernel cgroup v2 documentation](https://docs.kernel.org/admin-guide/cgroup-v2.html).

Child admission reconstructs a fresh allowlisted filesystem, private PID/proc/IPC and network view, cleared environment and tightly controlled descriptors. It does not inherit the root's `/state`, `/run`, sockets, `/proc` view, namespace handles, host home, SSH agent, Docker/systemd sockets or controller secrets. Source paths must belong to the admitted instance/Attempt; destinations must be normalized and reject traversal, overlaps and symlink escapes. Roles never submit arbitrary bwrap argv or unrestricted mount requests.

Existing `capsule.py` destination-prefix checks are not sufficient for an untrusted launcher interface (for example, a path containing `..` can satisfy a string prefix). Reuse inspected construction ideas behind a closed, independently validated supervisor interface; do not expose its general `--rw`/`--ro` inputs to agents. Pin image, tool binary and admitted mount/environment identities in each launch receipt.

Root networking is private. Expose only controlled operator/harness ingress and qualified outbound provider/Kata access through an instance-scoped gateway. Child networking grants only that role's approved provider/participant connections. Sharing the root network also shares its loopback services, so it is not the default worker policy. Network namespaces do not create outbound connectivity automatically; gateway/routing and authentication renewal must be qualified explicitly. Inherited host networking may be offered only as an explicitly weaker profile, which cannot pass the full multi-instance isolation contract.

The gateway enforces allowed destinations and scoped requests; deny arbitrary tunnels, root loopback/admin endpoints and cross-instance routes. Setting HTTP proxy environment variables alone is not enforcement. A local Kata fixture lives within the root; an external Kata deployment uses a unique project/WorkRef namespace and explicit scoped authority. Shared mutable tool/cache stores are excluded; identified immutable inputs may be shared read-only.

Provider credentials belong to runtime-only projections or a mediated provider connector, not general tool environments. App-server and arbitrary shell tools sharing one writable home, environment and process view would defeat that distinction. Qualify a nested tool-execution boundary or supported executor integration for all tool paths: shell, filesystem tools, MCP, patches, helper processes and background jobs. Merely wrapping some shell commands does not suffice. If credential/process separation cannot be proved, report the limitation and block the corresponding isolation claim.

The root supervisor may create children; workers may request bounded Executions through the controller but cannot choose arbitrary mounts or enter root/parent namespaces. Further child/tool capsules require supervisor-authorized plans, not worker-held root-launch authority. Capability disabling/seccomp must be chosen so required nested tool launches work while untrusted tools cannot rearrange authority. Do not blindly apply `--disable-userns` to a parent that must launch nested capsules. [Bubblewrap options](https://github.com/containers/bubblewrap/blob/main/bwrap.xml).

## Transport, UI and monitoring

Controller owns a trusted admin endpoint plus role-scoped participant bridges. A worker sees only its bridge; every request authenticates instance/role/Attempt and existing authority epochs. Agent-supplied sender or instance fields cannot select another principal. Controller runtime adapters use supervisor-held pipes or uniquely projected role sockets, reconnect by persisted runtime/thread-store binding, and reconcile ambiguous operations before repeating them. Prefer qualified local transport; the installed Codex CLI supports explicit Unix paths and stdio, and the official app-server interface documents both. [Official Codex app-server](https://learn.chatgpt.com/docs/app-server).

Use one instance-private tmux server/socket beneath the root, with the existing trial's lead/worker/controller/dashboard windows. A tmux socket is powerful operator control, not a worker observation channel; do not project it or inherit `TMUX`/`TMUX_PANE` into worker/lead tools. Views attach through trusted adapters to controller-owned executions. Native TUI attachment must be proven to observe the intended saved server/thread; if it launches another server or scheduler, use a controller-backed view instead and report that native attachment remains unsupported.

Use dedicated PTYs/relay processes and qualified session/terminal filtering to prevent child tools from injecting input into an operator's controlling terminal. Close nonallowlisted inherited descriptors before exec. Namespace/socket identities include the root incarnation, so attach/stop never resolve by a reused session title. [Bubblewrap security considerations](https://github.com/containers/bubblewrap/blob/main/README.md).

Host operator/outer harness enters through one explicit per-instance terminal/UI relay and authenticated controller ingress. Avoid globally publishing the internal tmux/app-server sockets. The harness drives the lead via this relay and inspects events/evidence independently; it does not gain worker scheduling or acceptance shortcuts. A browser dashboard may forward one explicitly admitted endpoint through the same host entry arrangement.

Keep the 15-minute alignment/context monitor and durable cursors under `/state/monitor`. It reads pinned visible runtime evidence via the gateway, observes context sizes/compactions, and routes corrections through the coordinator. Existing host monitoring remains transitional; do not silently move the current live lead thread or discard its receipts during rollout.

## Lifecycle and recovery

Admission sequence: validate instance/lock/storage/image -> establish owned supervision/cgroup -> launch root init -> establish private ingress/egress -> reconcile persisted state and runtime bindings -> start controller/projections -> admit role capsules -> attach views -> permit dispatch. First failure blocks later admissions; retain owned-resource receipts for cleanup. Launcher crash recovery must reconcile any root started before its acknowledgement was saved.

Persist host execution intents/observations in a separate supervisor-owned registry: instance/storage identity, host boot identity, root incarnation, stable launch operation, owned cgroup identity, observed process identity/start time and launch outcome. This is an execution journal, not another team scheduler or writer of controller task state. Supervisor death may release the lifetime lock while children are still alive or terminating. On restart, reacquire the lock and adopt or quarantine recorded survivors before any replacement launch; lock acquisition does not prove the previous root stopped. Unknown launch outcomes remain unresolved. Qualify death before/after acknowledgement separately from controller and root-init failures, and never signal a resource resolved only through a recycled PID/name.

The host service supervises root init, which independently owns worker Executions. Controller restart leaves root init and child jobs alive. Tmux/dashboard loss leaves services alive. `--die-with-parent` may bind the root to its dedicated supervisor, or a child to its independent execution supervisor; never tie worker lifetime to the replaceable controller or attached terminal. Match shutdown intent with this parent chain and retain unknown effects when observation is lost.

Root-init/root-service death is a different fault: qualify descendant termination/reconciliation, retain SQLite/workspaces/runtime histories/artifacts, rebuild ephemeral sockets, change root/runtime incarnation and do not dispatch replacements until survivors are accounted for. Full-instance stop closes admission, cancels/reconciles owned executions, flushes persistence, then terminates only that owned cgroup. Killing the root is not automatically a safe checkpoint. Backup import retains the baseline's authority fencing and external-effect reconciliation requirements.

## Qualification and implementation slices

1. Closed instance plan and host admission: duplicate-state refusal, path/alias/traversal tests, exact image/tool/env/mount receipts, explicit missing-feature errors. Public seam: start/inspect/stop instance and scoped launch/observe/cancel execution; CLI names are proposals.
2. Persistent root supervision: two concurrent roots with identical internal paths/ports/tmux session names, local SQLite reopen/rollback and whole-directory backup, controller restart with worker survival, root-death descendant accounting, host-supervisor restart with adopt-or-quarantine before launch and storage exhaustion containment. Use deterministic actual processes first.
3. Role/tool confinement and bridges: real separate Codex servers, isolated homes/stores/threads, all tool execution paths and inherited descriptors, provider renewal, scoped controller calls, denied parent/peer filesystem/process/socket/network access. Do not infer these from a bwrap smoke test.
4. Adapt the retained trial: a fresh instance, real Kata deployment/project isolation, two overlapping workers, exact-candidate verification/acceptance/conditional landing, questions/progress/context monitoring, detach/reattach and failure evidence. Run a second simultaneous instance; stopping one must leave the other intact. Prove shared remote landing separately if requested.

This extends tickets 02/05/06 without reopening accepted SQLite/Git behavior or redirecting the current ticket04 author. Freeze and retain existing product proofs. The coordinator should sequence new root-launch and qualification work after the current safe handoff; source/authority changes go through admitted revision rules. No live-worker relocation or adoption claim from this document alone.

## Observed feasibility and limits

On 2026-10-04 this host's bubblewrap 0.8.0 ran two independent root capsules concurrently, each launching a nested child with `--unshare-all`, a fresh root, read-only host `/usr` and only its Attempt directory projected to `/workspace`. All four workloads completed with exit 0. Root/child and A/B user/mount/PID/network/IPC namespace identities differed. Each child read its own A/B marker, could not see the parent's synthetic `/state/controller-secret`, and had only a route-table header (no routes).

This is a bounded model-free feasibility probe, not hermetic image, cgroup, credentials, SQLite, network-gateway, tmux, Codex runtime or all-tool isolation qualification. The host `/usr` was an explicit non-hermetic input. Standalone Insula currently excludes provider/tmux/credential integration and its reference runtime requires disabled networking; these additions belong in Corenius integration and, if needed, separately scoped Insula work, not an assumed existing product feature.

Independent separate-context critique identified parent-death coupling, inherited filesystem/FD/network authority, arbitrary bind traversal, shared runtime stores, cgroup allocation, tmux/terminal control and shared external targets. The requirements above incorporate each correction; none is claimed qualified by the smoke test. Pending product evidence is explicitly assigned to the four implementation slices.

The follow-up artifact review confirmed those corrections and identified two further gaps: storage/inode exhaustion and supervisor restart after lock release. Both now have explicit admission/recovery requirements and negative qualification scenarios above. No passing product isolation or recovery claim is made.
