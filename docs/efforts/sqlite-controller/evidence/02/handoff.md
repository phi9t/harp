# Ticket02 runtime handoff

Status: DONE_WITH_CONCERNS — implementation and deterministic gate complete;
independent Standards/Spec and branch verification remain owned by root. No
claim/resolve, SPEC/STATE update, push, landing, shared-ref mutation, model turn,
tmux change, PostgreSQL or Temporal suite was performed.

Implementation source commit: `de0f85f7cb8738d7c3674782230f128d47e236e1`.
Isolated base read: `c661309f372726a1987c6a03b2e4014fca2a94d8`.
Branch: `sqlite02/runtime`; tree:
`/data02/home/philip.yang/workspace/corenius-sqlite02-runtime`.
Ticket requirements and approved execution boundaries came from the canonical
02 ticket, SPEC/CONTROLLED-TRIAL, and root's
`.scratch/sqlite-controller/briefs/02-supervised-attempt-recovery.md` brief.
This handoff commit changes evidence only; tested implementation bytes are
identified individually in `.scratch/sqlite-controller/02/source-sha256.txt`.

The parent owns durable intent, claim, coordination, rehydrate/dehydrate and
independent review. This author was the sole writer in the assigned isolated
worktree, with no subagents. CONSTITUTION, agentic-engineering, domain guidance,
skill crosswalk, using-superpowers, TDD, subagent execution constraints and
verification-before-completion were read. Approved gated design/specification
and root's bounded task brief supplied the combined route; no duplicate plan or
approval ceremony was introduced.

## Implemented boundary

- Independent `corenius-attempt-supervisor CONFIG SOCKET` daemon owns actual
  `Child` executions, operation-keyed durable start records and a lifetime lock.
  Its records are effect observations, not another team scheduler.
- Configured controller automatically drains/reconciles all projects at startup
  and every 250ms between bounded requests. The operator secret is retained only
  in the controller; journal transactions end before any supervisor transport or
  process effects. Normal lead/outer clients issue admission/cancellation through
  the team API and never dispatch through supervisor RPC.
- `runtime:{kind:"reconcile"}` is an additional operator-only diagnostic seam.
  Only `fault-injection` builds recognize `CORENIUS_TEST_MANUAL_RUNTIME=1` to
  disable automatic drain for deterministic step evidence.
- Execution identities bind project, generated attempt, start operation, original
  authority epoch, pinned base/brief, random handle, supervisor incarnation,
  kernel boot ID, monitor PID/start ticks and actual namespace-init PID/start
  ticks/PID namespace inode. bwrap private `--info-fd` evidence is checked through
  independent kernel `/proc` identities. Retained immutable identities must match
  on adoption; the adapter queries current dispatch authority before any start.
- Lost replies resolve through the stable operation key; a recorded effect cannot
  be restarted. A missing acknowledged effect, unreachable supervisor, ambiguous
  setup or identity mismatch becomes unknown and retains capacity.
- Cancellation is its own journaled intent and revokes authority before effects.
  The supervisor kills its exact owned monitor group and waits for the retained
  namespace-init termination. PID1 teardown kills descendants even after setsid.
  Cancelling before start persists a tombstone that rejects delayed starts.
- Real bwrap Insula uses private home/workspace/scratch, cleared environment,
  isolated PID/network namespaces and private procfs. Only read-only `/usr` and
  the exact generated attempt directories are mounted. Controller DB/socket,
  supervisor storage/socket, host home, credentials, SSH agents, canonical Git
  metadata and other attempts are absent. Path aliases, symlinks and caller-
  selected profiles/mounts are refused.
- Trusted `WorkspacePreparer` and `serve_with_workspace` provide ticket04's
  workspace hook, including pinned base/brief identities. Returned mounts must
  match exact generated paths. Default workspaces are empty disposable fixtures.
- Socket production routes reject arbitrary terminal attempt observations and all
  operation transitions; runtime mode rejects every external attempt observation.
  `Controller::handle` without runtime remains the explicitly unqualified
  storage-only library fixture used by ticket01 tests.

## Public RED/GREEN and fresh checks

Retained ignored logs were explicitly staged with `git add -f`; they survive a
branch handoff. SHA256 receipts are in
`.scratch/sqlite-controller/02/evidence-sha256.txt`. The final gate log has only its
trailing empty line normalized for repository whitespace checks.

| Evidence | Command/result |
| --- | --- |
| `.scratch/sqlite-controller/02/red-runtime.log` | `cargo test --locked --offline --features fault-injection --test runtime -- --nocapture`; exit101, two actual public fixtures fail because independent supervisor executable is absent. |
| `.scratch/sqlite-controller/02/red-namespace-pause.log` | Same public runtime suite; exit101, missing kernel namespace-init identity and a paused prepared operation actually starts an effect. |
| `.scratch/sqlite-controller/02/red-autodrain.log` | Targeted `configured_controller_autonomously` public test with the actual periodic drain disabled; exit101, admitted worker never starts before the bounded deadline. This is regression RED, with real controller/supervisor/SQLite/filesystem effects and no fabricated supervisor replies. Exact disabled-service snapshot and source hashes are retained beside the log. |
| `.scratch/sqlite-controller/02/standalone-gate.log` | `sh tools/agents/checks.d/70-sqlite-controller.sh`; exit0, **31 tests**: retained20 public +4 process tests, and7 real runtime tests. |
| `.scratch/sqlite-controller/02/fmt.log` | `cargo fmt --manifest-path crates/sqlite-controller/Cargo.toml --check`; exit0. |
| `.scratch/sqlite-controller/02/clippy.log` | `cargo clippy --manifest-path crates/sqlite-controller/Cargo.toml --locked --offline --all-targets --features fault-injection -- -D warnings`; exit0. |
| `.scratch/sqlite-controller/02/build.log` | `cargo build --manifest-path crates/sqlite-controller/Cargo.toml --locked --offline`; exit0, normal non-fault binaries. |

The seven runtime cases cover two overlapping independently addressed processes;
controller SIGKILL/restart and heartbeat survival with the same retained handles;
identity-bound cancellation and independent host descendant disappearance; lost
start reply lookup and unavailable supervisor capacity retention; paused dispatch
and cancellation overtaking start; supervisor restart retaining unknown effects
and revoked capacity; symlink/stale-binding/profile injection/authentication
refusal; normal short-lived termination with retained identity; and automatic
controller-only admission/cancellation draining. Boundary probes use a real
listening host TCP endpoint and a real other-attempt namespace-init PID for
filesystem/proc/signalling denial. The stale worker grant fails team API mutation.

One additional short-lived test initially used an inadequate fixed30ms wait and
failed while its process was still running. That test harness timing assumption
was replaced by bounded observation polling; it is not claimed as product RED.

## Process identities, outcomes and resources

The final gate records real execution JSON, not terminal text as a substitute for
ownership. Examples from its two-worker proof:

- Final A: operation `e4acc0e4-39e7-4876-ba53-c88f91bd7ea3`, handle `1942c617-9e47-4ce1-bb1f-4bac66dd4c97`, monitor PID `2767816`/ticks `50806883`, namespace-init PID `2767820`/ticks `50806884`.
- Final B: operation `01117f16-c867-419b-b60f-f2f75740934a`, handle `6bdeabf7-1e32-4512-ab5f-206aa4a36335`, monitor PID `2767830`/ticks `50806885`, namespace-init PID `2767843`/ticks `50806886`. Both share supervisor incarnation `16ca4010-ba92-4542-b618-e554774846b0` and boot ID `6f48f645-3b26-40c9-93fb-db4190c7629b`; their distinct PID namespace inodes are `4026539959` and `4026540040`.
- Controller restart preserves both full immutable handles and advances a real
  worker heartbeat without another start. The outer test independently captures
  namespace descendants, including one with a new session, then verifies their
  host proc entries disappear while the other attempt remains running.
- Supervisor loss leaves its original nonterminal record unknown, even where the
  bwrap parent-death policy has killed the actual worker. Cancellation still keeps
  that slot occupied because reconstructed PID evidence cannot authorize release.

`.scratch/sqlite-controller/02/resources.json` records six final-run execution
identities, their monitor/init start ticks and independent kernel observations.
At observation, **zero owned execution identities remain alive**. Fixture roots
were removed by their owners after child shutdown. Retained resources are this
isolated worktree/branch, the standalone Cargo target cache and explicitly retained
raw evidence. No other agent or human worktree or process was cleaned.

## Reproduce and review

From this isolated tree or a fresh checkout of its final branch:

```sh
sh tools/agents/checks.d/70-sqlite-controller.sh
cargo fmt --manifest-path crates/sqlite-controller/Cargo.toml --check
cargo clippy --manifest-path crates/sqlite-controller/Cargo.toml --locked --offline --all-targets --features fault-injection -- -D warnings
sha256sum -c .scratch/sqlite-controller/02/source-sha256.txt
sha256sum -c .scratch/sqlite-controller/02/evidence-sha256.txt
```

For the auto-drain regression RED, use a disposable checkout, replace only
`src/service.rs` with the retained `service-autodrain-red.rs`, and run the targeted
runtime test. Restore the tested service before normal checks. Real bwrap tests
fail rather than skip when namespace setup is unavailable. Source/doc review is
against fixed base `c661309f372726a1987c6a03b2e4014fca2a94d8` and final branch HEAD.

## Limits and downstream seams

No live Codex or provider qualification is inferred. Profiles currently use null
stdin and retained stdout/stderr; they have no PTY or attachment client. Later05/06
must add a controller/supervisor-owned PTY or qualified observation attachment;
terminal disconnect must leave execution owned and alive. No attempt gateway or
provider connectivity was needed for these deterministic workers; those require
separate scoped gateway/credential proof before live execution. `/usr` is a
read-only host toolchain, not a hermetic/pinned dependency image.

Supervisor restart intentionally cannot recover owned `Child` handles and retains
unknown capacity. No operator text override, automatic cleanup, backup restore,
authority rotation or supervisor-loss release mechanism is claimed. Workspace Git,
Kata, submission/acceptance/ref updates and artifacts remain later tickets.
Independent root reviews must decide Standards/Spec and branch conformance before
any acceptance or integration.

## Repair round1: scoped review findings

Review base: `7d5c6235dba9ee3f5200820d2e1d154ff947d634`. This section
supersedes the original termination proof, which incorrectly accepted a zombie
namespace-init leader. Earlier receipts and source inventories remain unchanged
as historical evidence; use the new `repair-1/source-sha256.txt` inventory for the
repaired source.

The five scoped findings are repaired:

1. Reconciliation performs one supervisor lookup. A transport failure cannot
   fall through to another lookup and authorize dispatch. The actual absence
   branch retrieves current dispatch authority, and `dispatch_context` rejects
   a paused project. Existing execution observation remains available while paused.
2. The supervisor retains an identity-fenced namespace-init pidfd and uses it
   both to send SIGKILL and to observe whole-process readiness. Monitor exit,
   leader state Z, session membership and parent-death signalling cannot release
   a slot. Real zombie-leader/live-thread and separately addressed descendant
   pidfds now confirm termination before capacity is released.
3. Child ownership is installed immediately after spawn. Every subsequent setup
   error closes the launch gate, signals any qualified init pidfd and stops the
   owned monitor, with bounded termination polling. Cancellation uses that owned
   capability independently of projected record state. If whole-process proof is
   missing, its record/slot remains unknown even after physical disposal.
4. A trusted single-threaded `corenius-attempt-gate` holds PID1 on a private stdin
   channel. The supervisor samples/validates the kernel identity and pidfd and
   persists the execution before writing the one-byte exec permission. The gate
   then restores null stdin and execs the configured profile without changing PID
   or start time. Delayed identity sampling cannot race a short-lived workload.
5. Runtime-mode startup resolves existing database/socket files, checks their
   full canonical targets against the worker `/usr` mount, and validates parents
   before opening SQLite. A disposable mount-namespace fixture demonstrates the
   database symlink target is refused without creating SQLite contents or locks.

No PTY, provider routing, gateway, enum refactor, model call or broader broker
change was included. The adjacent gate binary must be installed with the
supervisor; Linux process-pidfd support is now an explicit runtime requirement.
Supervisor-loss reconstruction remains intentionally unknown and cannot release
capacity. Failed unqualified setup also retains its slot, despite disposal of
its actual owned child, until a future qualified recovery procedure exists.

### Genuine new regression evidence

All regression tests and test-only condition injectors were written and run
against the unfixed runtime logic before production repairs. Injectors only drop
an authenticated lookup transport, delay real identity sampling, or raise an
actual postspawn setup error; no supervisor identity, lifecycle state or process
termination reply is fabricated. The pre-fix source is reproducible from the
frozen review base plus `.scratch/sqlite-controller/02/repair-1/red.patch.gz`.
Its component inventory is `repair-1/red-source-sha256.txt`.

- `repair-1/red.log`: public `cargo test ... --test runtime repair_ -- --nocapture`,
  exit101. Paused first-lookup transport failure starts an actual effect; a
  workload executes before sampling; postspawn failure leaves a real owned
  child; and a database alias resolving inside `/usr` serves instead of refusing.
  The first zombie case passed while relying on parent-death signalling; the
  strengthened fixture below exposes why that was insufficient.
- `repair-1/red-zombie.log`: targeted `repair_zombie`, exit101. The real worker
  clears its own parent-death signal, makes its PID1 leader Z while another
  thread and a separate-session descendant survive, then cancellation releases
  capacity while its independently retained init pidfd is not ready. The fixture
  owns a pidfd cleanup guard, so the failed assertion disposes its own namespace.
- `repair-1/red-short-delayed.log`: targeted `repair_short_exit`, exit101. With
  a real200ms sampling delay, `/usr/bin/true` finishes before live sampling and
  remains unknown through the bounded observation deadline.
- `repair-1/green.log`: six focused public repair cases, exit0 after implementation.
- `repair-1/gate.log`: fresh standalone gate, exit0, **37 tests** (the prior31
  plus six repair regressions); real descendant pidfds also confirm readiness.
- `repair-1/fmt.log`, `repair-1/clippy.log`, `repair-1/build.log`: formatter check,
  strict all-target clippy and normal locked/offline build, each exit0.

The first gate implementation tried an unsupported `--preserve-fds` option;
that mechanical failure is retained in `repair-1/green-first.log`. The inspected
host bwrap supports ordinary stdin preservation, so the final gate uses stdin
only and restores null stdin at exec. It introduces no credential/service FD.

`.scratch/sqlite-controller/02/repair-1/resources.json` retains ten final-run
execution records and independent monitor/init kernel observations. Zero same-
identity owned processes remain alive. Setup-failure disposal is additionally
proved through the supervisor's empty real children list. The zombie/thread test
owns and confirms separately addressed descendant pidfds. Surviving resources
are this isolated worktree, Cargo target cache and explicitly retained evidence;
no foreign worktree, process, shared reference or user session was modified.

### Historical test chronology and provenance limits

The omitted classification is now explicit: supervisor-restart refusal,
symlink/stale-binding/profile rejection and the initial natural-exit test were
added as coverage strengthening of an already-GREEN original lifecycle slice.
No product change was required to make those added tests pass. They were not
separate before-implementation RED-led slices, and their passing tests are not
presented as retrospective TDD evidence. Root owns the policy granularity ruling
and independent Standards reassessment.

Actual edit/tool sequence and contemporaneous evidence:

1. `red-runtime.log` precedes the initial supervisor implementation; its two real
   fixtures fail because the executable is absent.
2. `green-runtime.log` records the original two-worker/restart/cancel/lost-reply
   lifecycle GREEN. The initial supervisor already had restart-to-unknown record
   loading, binding mismatch/deny_unknown_fields and generated-root symlink refusal.
3. Namespace-init identity/pause tests obtain `red-namespace-pause.log`, then GREEN.
4. `green-boundary.log` records three runtime cases after the trusted workspace
   seam and live host-network/cross-process probes. The two later cases had not
   yet been added.
5. Supervisor-restart and symlink/stale/profile cases are added; `green-full.log`
   is their first full run and passes five runtime plus the original24 cases.
   They require no product edit to satisfy the added coverage.
6. The natural-exit test is added; `green-short.log` is its first passing run.
   `green-short-source-sha256.txt` pins `supervisor.rs` to
   `920e98a27bc67823024595e673a7713ef7b54eefb203f34cb8c4d7762cc9cf53`, identical
   to the frozen reviewed supervisor. Its later fixed30ms assertion fails while
   the process is still running and is changed to bounded polling only. No
   product implementation change was made for that initial extra test.
7. Later autonomous-drain work changes runtime/main/service separately and
   records the previously documented drain regression proof. Repair636 now
   identifies the distinct short-profile sampling defect, for which the genuine
   new delayed-sampling RED and production gate repair above apply.

These original GREEN logs and the existing short-source inventory are explicitly
retained now, with `repair-1/chronology-log-sha256.txt`. There were no original
per-slice commits or complete contemporaneous source inventories for the initial
2/3/5-case runs. The short supervisor hash establishes component-byte identity,
not tool ordering or whole-tree identity. Ordering is the author's actual
recorded edit/tool chronology; log contents support the increasing test counts.
No missing historical source pin or RED is invented.

### Repair verification entrypoint

```sh
sh tools/agents/checks.d/70-sqlite-controller.sh
cargo fmt --manifest-path crates/sqlite-controller/Cargo.toml --check
cargo clippy --manifest-path crates/sqlite-controller/Cargo.toml --locked --offline --all-targets --features fault-injection -- -D warnings
sha256sum -c .scratch/sqlite-controller/02/repair-1/source-sha256.txt
sha256sum -c .scratch/sqlite-controller/02/repair-1/evidence-sha256.txt
```

Reproduce pre-fix regressions in a disposable checkout of the review base by
decompressing/applying the retained RED patch, then running the same targeted public tests.
Review the scoped repair diff from fixed base `7d5c623` to the delivered repair
HEAD. Independent review and acceptance remain root-owned; this is round1 of the
local three-round repair budget.

## Repair round 2: provisional pidfd qualification and portable fixture

Review base is exact repair1 source `d0bbb86269428d298ecd63ba89783b9295e22242`.
Scoped branch637 identified descriptor acquisition after namespace validation,
which could admit a descriptor for another process after init exit/PID reuse.
The namespace-init pidfd now remains provisional while the kernel descriptor
PID, PID namespace, namespace PID1 status, start identity and direct parent
relationship to the owned, unreaped monitor are validated. Monitor start ticks
must match the recorded owned child. Final process-level pidfd readiness must
still indicate liveness before the descriptor is installed as cancellation
authority. Failed qualification drops the provisional descriptor without
signalling it; cleanup closes the actual owned gate and terminates only the
owned monitor. A refusal with no qualified whole-process proof remains unknown
and retains its controller slot.

The Python zombie-leader fixture now receives `libc::SYS_exit` from the compiled
Rust target instead of the x86-64 literal60. The existing actual zombie leader,
live thread and detached-descendant cancellation regression still passes on the
current native Linux target. No AArch64 execution is claimed, and this fixture
portability repair is not presented as a new production RED-led behavior slice.

### Genuine bounded RED and GREEN

Evidence is explicitly retained at `.scratch/sqlite-controller/02/repair-2/`:

- `red.log`: targeted `repair2_` runtime tests exit101, two genuine assertion
  failures. The old qualification accepts both a real owned decoy pidfd and a
  real namespace-init acknowledgement from another owned monitor. Kernel stat
  and real pidfd readiness establish that the decoys are actual live processes.
- `red.patch.gz` and `red-source-sha256.txt`: pre-fix test/injection diff and
  contemporaneous source hashes, reproducible on the exact review base.
- `green.log`: the same two regressions exit0 after production qualification
  repair. Actual replies reject descriptor-target or monitor-lineage mismatch;
  each real decoy pidfd remains unready after refusal and cancellation.
- `gate.log`: `sh tools/agents/checks.d/70-sqlite-controller.sh` exits0 with
  all39 tests (original37 plus two qualification regressions). Final test
  assertions also inspect the refused effect's actual bwrap namespace metadata
  and independently wait for that private namespace to have no surviving
  processes. These supplemental disposal assertions require no further product
  change after the initial GREEN.
- `fmt.log`, `clippy.log`, `build.log`: formatter check, strict all-target clippy
  with fault injection, and normal locked/offline build each exit0.
- `source-sha256.txt`, `evidence-sha256.txt`: exact final crate-source and raw
  evidence receipts; verify with `sha256sum -c` from the worktree root.

The declared `fault-injection`-only seams are `CORENIUS_TEST_INIT_PIDFD_TARGET`
and `CORENIUS_TEST_INIT_ACK`. They substitute descriptor acquisition/acknowledgement
association using real fixture-owned processes, real Linux pidfds and actual
kernel namespace/stat observations. They model reassignment or misassociation
without forcing host PID reuse. This is not a claim of a naturally reproduced
PID-reuse race. No terminal reply, process identity or termination observation
is fabricated; the initial RED demonstrates improper acceptance as cancellation
authority, without deliberately signalling the live decoy through that bad
production authority. Fixture disposal signals only the fixture's separately
owned pidfds and owned children.

`resources.json` records independent post-run observations of36 unique
monitor/init kernel identities collected from retained RED/GREEN/full-gate
records: zero live processes with matching start identities remain. Inspection
uses read-only proc identity and process pidfd readiness, with no signals. Tests
additionally prove the supervisor's owned children list is empty and the actual
refused private namespace is disposed. Surviving resources are this isolated
worktree, its Cargo cache, and retained evidence. Foreign/user processes, other
worktrees, shared refs and tmux were unchanged.

Linux process pidfds, adjacent installed gate binary, and unknown/capacity-held
supervisor-restart ambiguity remain the previously reported limits. Independent
root review and acceptance remain pending. Root has separately directed a
subsequent bounded registry/topology phase for the owner amendment; this repair
commit does not claim completion of ticket02 or live model qualification.


## Owner-amended Codex registry phase

The independently qualified process core is unchanged. Source
`f77df6493f1090841bafa388b8b38b624a24571f` adds the durable controller-owned Codex
registry, paired shared/separate native topology, validated remote turn binding,
controller/app/executor recovery, native lost-ack/current-authority restrictions,
and actual turn interruption distinct from executor termination. All50 tests,
formatter, strict clippy and normal locked/offline build pass on the final source
pin. The detailed [registry handoff](registry/handoff.md) retains genuine native
RED/GREEN logs, honest source chronology/provenance limits, exact component/config
pins, final command exits and resource inventory.65 retained qualified execution
identities have no matching live survivors; all86 namespace disposal receipts
prove whole-process termination. No paid/live inference, UI gateway or assembled
baseline qualification is claimed. Root independently reviews and decides
acceptance; ticket02 remains claimed until that resolution.
