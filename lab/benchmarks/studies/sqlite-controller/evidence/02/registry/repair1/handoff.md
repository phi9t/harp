# Ticket02 registry repair1 candidate

Sole writer: `sqlite02/runtime`, `/data02/home/philip.yang/workspace/corenius-sqlite02-runtime`.
Frozen implementation source: `18f704c90fc8704a984fd507f01349e0d938b064`; crate tree
`b56c3d9d5ddbcd106d0cbe47665da09baf150b35`. Review base:
`2dcc4ec34bf7bdd74a6759bcd2f743361083e01c`; rejected source was
`f77df6493f1090841bafa388b8b38b624a24571f`. Parent root remains the sole review,
acceptance, coordination and integration owner. This repairs the four registry
findings in Spec/roborev639; it does not reopen the qualified process core or
claim ticket02 acceptance, source landing, live inference or downstream UI/tool
authority qualification. Original raw candidate evidence is preserved unchanged. The original handoff and
manifest are also retained verbatim in `raw/original-*`; the existing handoff is
appended and its current manifest hash is refreshed.

Classification: **MODEL_FREE_ACTUAL_PROTOCOL**. Both topology modes use the real
pinned native app-server, companion and isolated executors. Deterministic local
Responses/SSE bytes cause actual native shell and patch effects. No model/upstream
calls, production credentials, host ptrace changes, user tmux/process mutations,
other checkout writes, subagents or shared branch rewrites occurred.

## Four scoped changes

1. Successful later turn acknowledgements and exact lost-ack history confirmation
   now commit their operation receipt and binding turn adoption in one short SQLite
   transaction. The typed `current_turn` ID/status accompanies `turn_id` in that
   transaction. Native RPC/history reads remain outside the transaction. Real
   feature-gated exit79 faults occur immediately after committing each receipt,
   before the remaining in-memory/post-dispatch work. Independent read-only SQLite
   snapshots show the exact actual new native ID and in-progress typed status at
   this cut; restart, exact client-message uniqueness and public interruption of
   that same active turn pass without a second launch. Existing current issuer,
   actor epoch, unknown-start fencing and cancellation boundaries remain enforced.
2. Native-store receipts have their existing `native-store-<operation>.json` file
   namespace parsed as a strict distinct receipt type. Operation, execution and
   generation identities, numeric filesystem identity format and exact linkage to
   the retained execution are validated. Unknown arbitrary JSON, malformed or
   mismatched receipts and symlinks still refuse startup. Clean canceled records
   reopen in both topologies. Supervisor ownership loss with live records becomes
   unknown, retains capacity and refuses repeated starts with the same handle;
   no child/pidfd ownership is reconstructed from PID text.
3. A persisted empty `thread_bound` identity reconnects only after its retained
   creation receipt matches the exact current qualified app incarnation and real
   native `thread/loaded/list` contains that ID. It uses the surviving loaded
   identity instead of ID-only resume, and never recreates a thread. An actual
   identity-fenced owned-app kill/restart proves a changed incarnation retains the
   empty identity as unknown, blocks input eligibility, holds both slots and
   creates neither another thread nor a first turn. Missing loaded identity is
   also explicitly refused by the same guard; no synthetic loaded-list reply is
   used as evidence.
4. WebSocket activity drains only the exact target thread, retaining other thread
   notifications within the existing bounded queue. Unscoped global notifications
   do not enter actor projections. A deterministic barrier releases B's actual
   user-message, shell, patch and assistant completion notifications while A is
   next to drain the shared connection. Received-frame snapshots prove the native
   B notifications existed before the drain; the public B projection retains the
   exact received completion frames and contains no foreign-thread events. The
   separate-connection mode is a positive control. No raw server capability or
   dynamic response authority is exposed to another actor/UI.

## Genuine chronology and commands

All test commands below use `cargo test --locked --offline --manifest-path
crates/sqlite-controller/Cargo.toml --features fault-injection --test codex FILTER
-- --nocapture --test-threads=2`. A Rust repair fixture invokes both real native
topology modes independently, retaining both outcomes even when one fails.
Fault hooks are explicit test-feature crash/barrier/ack-loss injections on owned
resources, never invented successful receipts, identities or terminal outcomes.

| Retained log | Filter | Exit and observed outcome |
| --- | --- | --- |
| `raw/red-all.log` | `repair_` | 101; original 5 tests failed, 230.62s. The combined supervisor case failed at canceled reopen before reaching its live branch. |
| `raw/red-confirmed.log` | `repair_` | 130; interrupted repeat after a fixture-edit assertion failed. Not credited as a terminal test or qualification. |
| `raw/red-frames.log` | `repair_` | 101; strengthened real received-frame oracle and split canceled/live cases: 6 failed, 302.43s, both topologies. |
| `raw/green-focused.log` | `repair_` | 101; 3 passed/3 failed, 301.28s. Adoption and one client message succeeded, but fixtures incorrectly required interrupt confirmation on a completed turn and read a stale restarting listener immediately. |
| `raw/green-focused-fixture-corrected.log` | `repair_` | 0; 6 passed, 251.00s. An actual active turn and bounded service readiness retry correct those fixture assumptions; no product changes were needed for the corrections. |
| `raw/red-typed-projection.log` | `confirmation_survives_binding_crash` | 101; 2 failed, 65.36s. Read-only exact-cut snapshots genuinely exposed new `turn_id` with old typed `current_turn.id` in both modes and both receipt paths. |
| `raw/green-typed-projection.log` | `confirmation_survives_binding_crash` | 0; 2 passed, 67.71s after atomic typed adoption refinement. |
| `raw/green-empty-incarnation-negative.log` | `changed_incarnation_stays_unknown` | 101; fixture Python local import shadowing failed before sending the owned-app fault, 47.19s. |
| `raw/green-empty-incarnation-negative-corrected.log` | `changed_incarnation_stays_unknown` | 130; interrupted repeat after an edit assertion failed. No qualification credited. |
| `raw/green-empty-incarnation-negative-fixed.log` | `changed_incarnation_stays_unknown` | 101; actual replacement and public unknown behavior passed, then fixture used a missing JSON key instead of nullable lookup, 53.50s. |
| `raw/green-empty-incarnation-negative-final.log` | `changed_incarnation_stays_unknown` | 0; 1 passed, 61.18s after fixture-only nullable lookup correction. |

`red-source`, `red-confirmed-source`, `red-frames-source`, `green-focused-source`,
`red-typed-projection-source` and `green-typed-projection-source` JSON inventories
and complete crate archives were captured before their corresponding commands.
The original first failed GREEN has no contemporaneous whole-crate snapshot;
that limit is explicit. Supplemental receipt corruption/symlink and changed-app
incarnation controls strengthened already implemented guards without new product
changes; their passing tests are not manufactured retrospective RED proof.
`gate-final-source.{json,tar.gz}` pins the final complete crate before the fresh full gate.
The prior failed618 full gate keeps its original `gate-source` snapshot.
Source/config/test bytes stay frozen through that gate and delivery.

## Full-gate startup diagnosis and bounded repair

The first complete gate on frozen `61855127702745ae66f588b43f441968213dad01`
stopped with exit101: native16/17 passed, one existing
`unknown_native_start_refuses_new_turn_until_reconciled` failed after its own
lost-ack/unknown-start/history checks while waiting for a later lead thread.
`raw/gate-tests.log` retains the 596.63s native phase; `raw/gate-commands.json`
retains the 597.55s command and exact UTC start. Tests after that native failure
were not run by this attempt. Its pre-command25-file inventory remains
`raw/gate-source.json` (SHA256
`22dc86ede145bb892c46c3b91e36d96a72502c3de6c0fece334f343b82a69d6f`).
Root independently reported a complete57-test/fmt/clippy/build PASS on the same
618 source; root owns those separate receipts. Neither outcome is omitted or
substituted for the required final-source gate.

The retained actual controller stderr identifies a disconnected lead environment:
its native exec-server connection failed with `Handshake not finished`. Native
process identity was already sampled but its listener had not completed a ready
handshake. The adapter saved registration incarnations after environment/add,
then only polled disconnected status on subsequent ticks. The unchanged-source
minimized original test passed 87.45s in `raw/diagnose-full-gate-lead.log`, so the
ordinary command did not reliably reproduce the startup window. Root explicitly
authorized a deterministic real-owned startup fault and minimal readiness guard
within this same registry repair1.

A feature-only hook, after native PID/start/namespace/executable/capability and
pidfd validation, SIGSTOPs only the newly owned lead executor through that pidfd.
The fixture independently opens its positively owned descriptor, validates actual
start/namespace/stateT and nonterminal pidfd, holds it over3s, samples the public
registry while it remains stopped, then signals SIGCONT on that retained
identity-fenced descriptor. It never signals a foreign/guessed host PID, creates
fake native readiness/termination replies, or releases the pause early.

| Log | Actual outcome |
| --- | --- |
| `raw/red-exec-readiness.log` | 101,79.92s; test selector incorrectly required a lead-only field on worker rows. No product RED credit. |
| `raw/red-exec-readiness-corrected.log` | 101,80.20s; selector was nullable, but the fault selected an existing worker before the intended lead. No product RED credit. |
| `raw/red-exec-readiness-owned.log` | 101,70.82s; intended actual lead was stateT in both modes and native environment/add was already confirmed while stopped. Genuine production RED. |
| `raw/green-exec-readiness.log` | 0,91.27s; stopped for 3.014/3.017s, zero registration while stopped, then one confirmed native registration and public lead/thread/authority/cancellation proof in both modes. |

Every readiness RED/coverage GREEN source inventory and archive was captured
before its command (`red-exec-readiness-source`, `-corrected-source`,
`-owned-source`, `green-exec-readiness-source`). The hooks do not alter normal
production builds. The minimal production guard requires an actual authenticated
WebSocket upgrade through the qualified executor's own Unix bridge and exact
native start fence before the new/restored fixed-binding registration branch.
It uses200ms send/read timeouts and a fixed upgrade-read deadline below the
supervisor/client budgets. Unready transport returns through existing unknown/
UI-blocked retry; no environment/add is dispatched before readiness. The check
runs at registration/restoration, not on every established tick. It does not
repeat or redefine previously disconnected registrations, change native effect
deduplication semantics, or guarantee that a listener cannot fail after a
successful readiness probe. Existing current authority checks stay in force.

## Exact final source pins

| File | SHA-256 |
| --- | --- |
| `crates/sqlite-controller/src/codex.rs` | `00a98f0b767b949e09499edce8c6e1b8428cd7a31317c8ccd22e6d88df9074e5` |
| `crates/sqlite-controller/src/native_ws.rs` | `da208123f485c8f66e5ac57fb5f73a3c18e61eb36337944766673225f658f6c4` |
| `crates/sqlite-controller/src/supervisor.rs` | `92c7036e09d7b6e974f511b32f0c7daa41f1a58f2931fcc3641919534b2b1a8f` |
| `crates/sqlite-controller/support/codex_insula.py` | `d749ad030f6ac45ae70d00b6989dccee7ca1e5cccc5f12cb560b770f0a62ec8f` |
| `crates/sqlite-controller/tests/codex.rs` | `7deb58374c0cc20ae0b1072525614e922dad1c65ce524c2cfee1a5a36eb43a51` |
| `crates/sqlite-controller/tests/support/native_registry.py` | `4756ac79af13868494cec47aaaa3082ce18262c430a7ed96e4e439129748ab3d` |

The full inventory includes Cargo.lock, all controller sources, support scripts,
tests and README. Native binary/companion/catalog/generated schema pins are
unchanged from the original registry handoff. The trusted launcher hash is
unchanged; each real fixture retains actual per-root profile/config bytes and
both component qualified identities in `exact_native_profile_configs` and
`actual_bound_registry`, plus received frames, receipt IDs and kernel disposal.

## Terminal gate and resources

Final frozen source `18f704c90fc8704a984fd507f01349e0d938b064` passed all 58
checks:1 native wire unit,18 native integration,4 process,20 public/controller
and15 runtime tests. `raw/gate-final-tests.log` retains the full output; command
exit0,670.52s, with explicit `--test-threads=2`. Formatting, strict all-target
Clippy with fault-injection and normal locked/offline build each exited0 in
sequence after the test process terminated. `raw/gate-final-commands.json` keeps
actual commands, UTC starts, durations, exits and frozen-source equality.

Final25-file pre-command inventory SHA256:
`1981b8b989cec65e0a9988c2dcd59ca4bf1fd79aa487529b1e39972604c3273f`;
archive SHA256:
`755c9fff072870a5383329d55ffdf3b90ead1d9cf57ac8cd85f9853f2ffc3ab8`.
Both live files and the final source commit match the complete inventory; final
evidence delivery commits do not change the crate tree.

`raw/gate-resources.json` and its retained read-only observation script account
for 1599 positively retained owned identities across this repair's complete
logs, with 0 matching live processes; all 558 whole-process pidfd disposal
receipts report termination. All 131 listed native fixture temporary roots are
absent. The audit samples actual pidfd/kernel state or observes identity
replacement/absence, never treats leaderZ as whole-process termination and never
signals/inspects unretained foreign processes. Fixture disposal uses retained
qualified owned namespace descriptors. Explicit provenance limit: interrupted
cargo runs that never emitted their fixture resource identities have no complete
retained inventory and are not claimed as such. They are not qualification gates.

Independent branch640 subsequently identified three further medium recovery
boundaries: confirmed initial turn adoption before empty classification after
app restart, pagination of actual loaded identities, and disposal of events for
retired bindings. Root dispatched these as registry repair2/3. They remain open
at this clean R1 evidence handoff; the passing58 gate and initial four-finding/
startup repair do not claim ticket02 acceptance. No source changes for round2
occurred during this gate.


Reproduce from the final clean candidate:

```sh
cargo test --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml --features fault-injection -- --nocapture --test-threads=2
cargo fmt --manifest-path crates/sqlite-controller/Cargo.toml --check
cargo clippy --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml --all-targets --features fault-injection -- -D warnings
cargo build --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml
```

## Precise retained limits

This remains Linux/pidfd/bubblewrap plus the adjacent pinned native-package gate,
with protected supervisor/auth/store roots on local trusted storage. A supervisor
restart cannot reconstruct owned children or cancellation descriptors; ambiguous
execution remains unknown with slots held. The bounded loaded-thread lookup is
sufficient for this fixed two-worker/lead baseline; missing identity fails closed.
Native completion and native interrupt disposition are independent of executor
whole-process termination and task success. The event projection is a scoped
bounded activity snapshot, not a durable tool-call gateway or transcript archive.
Shared app raw bearer/control remains controller-only; native UI ingress, filtered
attachment/PTY, dynamic tool response authority, assembled provider behavior and
paid/live Codex remain downstream05/06 qualification. No such claims are inferred
from these model-free repairs.
