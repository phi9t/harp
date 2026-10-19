# Independent Standards review: FAIL

Frozen source `ae4e6fed22d08eab45cdbcddf6e472579a6899a8`, tree `9ec2a1d066dbe1a6a1872849458dcda6ac7006e3`, base `291f04a5abd9274e4fd0034fd36927b0a4c4d236`. All43 admitted Git blobs and SHA256 inputs matched before and after all probes; safe own archive SHA256 `f57d0c3b73d7adc4d56252b488236c0b1c5d3cf2317c74bb49177aad03f48801`. Admission SHA256 `445d1ec1bde81b9963784ee6048ab0b0990cd8ee3d2ff93fbe540f7a2e219228`; brief SHA256 `fa798481b1c4dfec9b39a7cf5e0fe66503ae20884e690031a548be47521d70fb`. Full ticket05 crate delta:23 files,+5522/−49. Original six findings resolved; three new P2 findings remain. This is independent review, not controller acceptance or root full-gate success.

Review owned by `/root/sqlite05_standards_review` in `/data02/home/philip.yang/devx/tmp/sqlite05-standards-final-ae4e6fed-wfohrlnm`. Source immutable; only own appended cfg(test) fixtures/targets used. Both probe maps prove all43 production prefixes unchanged. Read canonical policy/spec/trial/owner832 design and reviewed all changed client/control/gateway/monitor/native_ui/native_ws/ui_gateway/schema/test and lib/codex/query/service/Git paths; no cosmetic findings. The remaining defects expose incomplete response/retry/projection contracts at the module boundary.

## STD05-007 (P2): Return a native start response after history reconciliation

Location: `crates/sqlite-controller/src/native_ui.rs:402` at frozen commit. Related: `crates/sqlite-controller/src/native_ui.rs:405`, `crates/sqlite-controller/src/codex.rs:1353`.

Requirement: SPEC.md:58,78 and ticket05:12,21 require compatible native controls and durable visible dispositions after lost acknowledgement/reconciliation.

Observed: A saved codex_turn whose lost start acknowledgement is reconciled from typed native history is confirmed with internal {turn_id,thread_id,...} observation. ui_input returns it unchanged as successful turn/start response. The native consumer expects {turn:{id,...}}; turn is absent despite the confirmed accepted turn.

Independent evidence: Actual ui_input caller on real SQLite prepares the retained operation; actual reconcile_turn_ack confirms the exact operation/client ID/text from typed history. Exact retry returns internal observation with turn_id=accepted-turn and no turn object; operation count remains one. A normal native-shaped acknowledgement is returned correctly, and changed arguments are refused. Log `independent-edge-probes.log`, marker `EDGE_HISTORY_UI_RESPONSE`.

Correction: Normalize reconciled saved-start observations into the same version-pinned native turn/start response as an observed acknowledgement, preserving accepted turn identity and saved command semantics.

Limit: Qualified owned binding and declared lost-ACK/history fixture; native drain held by existing fault-injection flag. No native subprocess or wire consumer was used for this edge discriminator.

## STD05-008 (P2): Replay an exact retained steer after the current turn advances

Location: `crates/sqlite-controller/src/native_ui.rs:305` at frozen commit. Related: `crates/sqlite-controller/src/native_ui.rs:308`.

Requirement: SPEC.md:30,46,58,78 and ticket05:21 require stable retained retries and reconciliation while checking current authority; a new stale request must still be refused.

Observed: The expectedTurnId/current-turn guard runs before saved control_ui_commands lookup. An already admitted, confirmed steer with unchanged stable ID/arguments and unchanged actor/binding/epoch cannot return its retained receipt after a later turn becomes current.

Independent evidence: Actual ui_input prepares one codex_steer. Exact retry with retained native response succeeds before advancement; after current_turn and turn_id advance it fails native UI expected turn mismatch. No second operation is created. A fresh stale request is independently refused as expected. Log `independent-edge-probes.log`, marker `EDGE_RETAINED_STEER_REPLAY`.

Correction: Validate saved identity/arguments/current authority and replay admitted work before applying current-turn admission checks to new requests. Keep fresh stale, foreign, superseded, and changed-argument refusals.

Limit: Real controller command/receipt/caller with qualified owned binding and declared confirmed observation/turn advancement. No native subprocess used for this edge discriminator.

## STD05-009 (P2): Bound the control-center question projection to its shared transport

Location: `crates/sqlite-controller/src/control.rs:354` at frozen commit. Related: `crates/sqlite-controller/src/control.rs:356`, `crates/sqlite-controller/src/client.rs:36`, `crates/sqlite-controller/src/service.rs:154`.

Requirement: SPEC.md:32,58,80 and ticket05:12,20 require usable shared dashboard/lead projections, retained large text, questions and uncertainty.

Observed: control_center selects every question including full text/answer without a result budget. Valid retained questions accumulate past the shared 4 MiB client response limit, making the whole dashboard and gateway projection unavailable.

Independent evidence: Public Controller.handle created one small plus 70 valid 65,536-byte questions in on-disk SQLite. Actual framed response is 4,606,124 bytes versus 4,194,304-byte limit. Actual service::serve_request and client::call over owned Unix sockets return controller response bound/framing for dashboard transport and gateway corenius_query. Small projection succeeds and single question_input retrieves all 65,536 bytes. Log `independent-edge-probes.log`, marker `EDGE_CONTROL_CENTER_RESPONSE_BOUND`.

Correction: Return bounded question descriptors/references or pagination with explicit continuation/availability; keep exact full text available via question_input. Account for all projection fields within the shared response budget.

Limit: Actual product service/client/controller/gateway functions and owned Unix sockets; no browser or separate MCP subprocess was needed to establish the shared transport failure.

## Original six findings

| ID | Disposition | Exact current location and proof |
|---|---|---|
| STD05-001 | RESOLVED | `src/git.rs:359`. Generic Git command route restricts observer principals before effect preparation. Unchanged repaired Git SHA from prior924 review retains its independently executed configured-Git denial evidence; fresh public observer-authority projection controls passed. No need to repeat unaffected Git suite. |
| STD05-002 | RESOLVED | `src/gateway.rs:237`. New native calls require active observed exact thread/turn and current or admitted in-flight correlation; completed retained response replay is allowed under current binding/epoch. Fresh unit and actual both-topology early-ACK/runtime tests passed. |
| STD05-003 | RESOLVED | `src/native_ws.rs:492`. Owner RPC handles only matching-thread tool calls and queues peer calls for their own routing. Real socket test and actual combined native interleaving in both topologies passed. |
| STD05-004 | RESOLVED | `src/gateway.rs:229`. Stored invocation arguments are compared even with pending NULL response, before recovery dispatch. Changed pending arguments refused; same pending and completed replay controls passed. |
| STD05-005 | RESOLVED | `src/codex.rs:1443`. Local stale-turn failure is checked before applying admission. Actual dispatcher SQLite second-write abort leaves prepared/prepared; reopen plus ordinary reconcile_turn_ack/dispatch_prepared/prepare_delivery yields old failed, fresh different codex_turn prepared, question prepared, total two. Helper atomicity, early ACK, and correction ACK remain intact. This resolves prior924 independent caller defect; new replay defect STD05-008 is separately identified. |
| STD05-006 | RESOLVED | `src/monitor.rs:88`. Bounded nonblocking nofollow regular-file retained reader denies FIFO/symlink/oversize/corrupt/missing inputs without credit; fresh tests passed. Private atomic publication and real kernel short-write discriminator also passed. |

## Six repair controls

- REPAIR1-01: **PASS**. Actual caller fault leaves pre-effect prepared/prepared; ordinary reopened recovery admits a different fresh question delivery without blindly retrying applying/unknown external effects. No native process-kill claim for SQLite-fault probe. Evidence: `units.log`, `independent-probes-final.log`.
- REPAIR1-02: **PASS**. Missing/corrupt/FIFO/symlink/oversize/incomplete full inputs are uncredited. Independent all-bytes-present metadata-hole fault is refused with zero assessments/credits and unchanged revision; ordinary metadata deletion is first proven protected, then declared corruption is injected only in owned DB. Filesystem validation outside writer; immutable dependency rebind inside transaction. Evidence: `units.log`, `independent-probes-final.log`.
- REPAIR1-03: **PASS**. Actual pinned native interrupt in shared and separate topologies accepts native shape with turnId stable identity; exact cached replay works, stale/foreign requests refused. Evidence: `native-combined.log`.
- REPAIR1-04: **PASS (bounded case); extension FAIL STD05-007/008**. Genuine immediately persisted active saved codex_turn with owned hold replays exact original native-shaped response, unchanged counts21/21 shared and17/17 separate; changed arguments refused. Lost-ACK history response and retained steer after later-turn advancement fail separately. Evidence: `native-combined.log`, `independent-edge-probes.log`.
- REPAIR1-05: **PASS delivery; projection extension FAIL STD05-009**. 10-question aggregate with65,536-byte item and8×16,000-byte items plus later small question resolves in two actual native deliveries in both topologies. Exact full reference digest/text retrievable; foreign task refused. Unbounded dashboard projection separately fails. Evidence: `units.log`, `native-combined.log`, `independent-probes-final.log`, `independent-edge-probes.log`.
- REPAIR1-06: **PASS**. Private create_new/no-follow stage, write_all+fsync, renameat2 NOREPLACE and directory fsync. Awaited owned partial-stage child exits86 before publication; reopen/concurrency preserves one input and unknown orphan. Independent RLIMIT_FSIZE1024/SIGXFSZ-ignore real kernel write16384→1024 thenEFBIG leaves no final digest, removes owned stage, and full16384-byte unrestricted retry succeeds. No power-loss/SIGKILL proof inferred. Evidence: `units.log`, `independent-probes-final.log`, `kernel-short-write.strace`.

## Actual commands and exits

Each named receipt retains exact argv/cwd/environment, PID/start identity, elapsed time and log SHA256. Actual completed runs:

| Receipt | Exit | Seconds | Meaning |
|---|---:|---:|---|
| `independent-edge-probes-receipt.json` | 0 | 19.193 | observed defects independently reproduced; assertions of bad behavior passed, so review remains FAIL |
| `independent-probes-corrected-receipt.json` | 101 | 1.761 | reviewer fixture setup failure retained, corrected later; not a source finding |
| `independent-probes-final-receipt.json` | 0 | 1.503 | focused behavioral qualification; not whole repository gate |
| `independent-probes-receipt.json` | 101 | 17.851 | reviewer fixture setup failure retained, corrected later; not a source finding |
| `independent-sealed-dependency-receipt.json` | 0 | 1.345 | focused behavioral qualification; not whole repository gate |
| `initial-confirm-default-receipt.json` | 0 | 61.025 | focused behavioral qualification; not whole repository gate |
| `native-combined-receipt.json` | 0 | 140.183 | focused behavioral qualification; not whole repository gate |
| `public-monitor-controls-receipt.json` | 0 | 0.849 | focused behavioral qualification; not whole repository gate |
| `units-receipt.json` | 0 | 18.209 | focused behavioral qualification; not whole repository gate |

Core commands used `cargo test --offline --locked --features fault-injection` with own `CARGO_TARGET_DIR`,2 build jobs and1 test thread. Pristine `--lib`20 tests; native `--test codex actual_native_control_races_and_authenticated_correction` with UI_FIXTURE/UI_DRAIN_RACE/TOOL_INTERLEAVING/QUESTION_BYTES=1,UI_REPAIR=both; default `repair2_initial_turn_receipt_survives_app_restart_before_adoption`; public `--test control_center`; independent `--lib standards_`/sealed-dependency selection; separate edge `--lib standards_edge`. The edge suite exit0 asserts defects, not repair success. No unaffected generic healthy suite repeated.

Actual combined native test passed shared and separate topologies and retained four full native fixture archives for combined/default runs. It proves native delivery, active saved-kind replay, interrupt/replay/denials, early ACK/correction, peer routing and owned reconnect cases in the pinned fixture; no paid/live LLM claim.

Author68–71 records/maps/archives/logs independently verified as exact43 inputs and actual exit0; see `author-evidence-verification.json`. These are implementer evidence, distinct from own execution. Author64 actual exit101 remains retained even though filename claimed green: its old-operation JOIN became stale after recovery attached the question to fresh delivery. Author65 queries old operation and question independently. Own dispatcher probe independently checks before-abort states, reopen, old failed/new distinct prepared codex_turn and question prepared.

Initial-confirm measurement ruling d11494af5ec79471967a8a23298da8d1476dfc6b4c25b01d24243d5f3e30b0de permits only test-driver waiting for the original exact turn/clientId predicate in the same10-second budget. Verified before0bfe72fb…/after8b567d95… byte-exact diff; after equals frozen driver. Default actual both-topology run exit0. Provider question fixture now fetches immutable question_input references and still performs explicit query/ACK/answer; this follows the changed large-text boundary. No older failure attribution.

## Evidence retention and limits

All55 recorded positively owned process/namespace identities are absent at final check (`owned-process-cleanup-final.json`); no broad process signal, user-session mutation or shared target use. Edge Unix socket threads were joined. Native cleanup archive dispositions identify killed/observed owned processes and namespaces; process termination does not erase controller uncertainty. Manifest `evidence-sha256.json` indexes all retained readable/structured reports, commands/logs/source maps/diffs/probe sources, policy sources, author evidence and native archives with actual sizes/digests.

- Root full gate80070, Spec review, branch review and acceptance are separate; no result for them is claimed here. Candidate remains unaccepted.
- No paid/live LLM, user session, tmux or author/root process/target mutation. Product permanent recurring runner/live outer qualification stays06.
- Edge native response/retry probes invoke actual product private caller with declared qualified owned binding, actual saved controller commands/history reconciliation, and no native subprocess. They establish caller contracts, not extra wire/live qualification.
- Publication source crash child used actual exit86 after partial stage, not a power-loss/SIGKILL test. Its PID was not emitted; awaited output/exit and recorded process checks bound this limitation.
- Two reviewer setup failures are retained. First RLIMIT also affected inherited regular stdout; pipe setup corrected. Second immutable dependency DELETE was correctly denied; declared owned-only metadata corruption then tests unavailable publication. Reviewer-only unused_mut warning does not characterize frozen source strict Clippy.
- Original gate36 stays31PASS/2FAIL, exit1,1101.68s; original async-cause/initial-confirm raw historical evidence limitations remain unresolved. No retroactive success/cause attribution. Original02/04 final caps3/3 and original04nine findings unchanged.
- Initial-confirm driver-only authorized change waits original exact turn/clientId predicate within same10 seconds and retains final assertion. Byte-exact before/after proof and default actual both-topology success do not establish cause of older gate36 failures.

No source fix, commit, status change or acceptance/landing was made. Required next review scope: exact repairs for STD05-007/008/009 with positive authority/stale-request controls and frozen committed evidence reconciliation.
