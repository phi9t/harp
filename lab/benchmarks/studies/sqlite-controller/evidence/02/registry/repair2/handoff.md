# Ticket02 registry repair2 candidate — full gate passed, review failed

Sole implementation writer: `sqlite02/runtime` in
`/data02/home/philip.yang/workspace/corenius-sqlite02-runtime`.
Review base `cbe7854b59898f9750d1478d2277584c7eca7b42` preserves the R1 evidence
and source `18f704c90fc8704a984fd507f01349e0d938b064` unchanged. Frozen R2
source: `6f55bc67624b2399c3430fe2c5e003baeb4c22e0`; crate tree
`9aead302dc0826d69b91112e8e5dca228d9a90cb`. Parent root owns review, acceptance,
canonical metadata and integration. This addresses the three branch640 findings;
it claims neither ticket02 acceptance nor source landing/live inference/UI gateway
qualification. No other checkout, canonical SPEC/STATE/ticket or shared branch was
edited. No helper agents, user tmux/process mutations, host ptrace changes,
production credentials or upstream model calls were used.

Classification: **MODEL_FREE_ACTUAL_PROTOCOL** for the public native regressions.
Real pinned Codex app-server/companion/private executors run in the existing shared
and separate Insula topologies. The harness admits/cancels through the controller;
the configured daemon owns runtime scheduling and reconciliation. Explicitly named
native padding, history observation and thread-name setup calls use only the owned
fixture controlplane; they are not controller assignments, agent turns or a second
worker dispatcher. Every setup request and success oracle derives from actual
native responses. Declared Unix-pair transport fixtures separately test malformed,
repeated and exhausted cursor responses; those are not native qualification.

## Three changes

1. After current authority and qualified executor/app/store checks, initial
   confirmed `turn/start` receipts are adopted before null-turn empty classification.
   Exact receipt thread/turn identity is checked; `turn_id` and typed `current_turn`
   are saved together. The retained acknowledgement alone marks current status
   unknown; fresh actual resumed native history supplies current status. Real
   exit79 immediately after the confirmed receipt, followed by identity-fenced
   native app death/restart, restores the same original thread, turn and one
   client message in both modes. No thread or first turn is recreated. Native
   resume leaves input blocked until explicit remote restoration/validation;
   the proof establishes exact adoption/history and safe cancellation, not
   unrestricted view input. Genuine empty changed-incarnation unknown/no-replay
   protection remains in the full gate.
2. Empty-thread recovery follows actual loaded-thread pagination until exact
   identity is found or the valid list is exhausted. All pages share one absolute
   eight-second deadline; more than 10,000 IDs or 100 pages returns precise
   uncertainty. Data/cursors are validated and repeated cursors rejected. Actual
   setup creates 101 empty threads before and 101 after the controller-owned empty
   thread. Native pages prove its ID on page 2 in both modes; only two controller
   attempt bindings exist. Malformed/repeated/exhausted scan checks are explicitly
   declared transport-unit fault coverage. They do not claim an actual native
   malformed reply or a separate native absent-thread experiment.
3. Retirement persists actual turn stop disposition before local observation
   withdrawal. Qualified native `thread/unsubscribe` is journaled before external
   RPC with exact server/store/incarnation/thread and connection UUID; unknown
   acknowledgement retries only this supported idempotent connection-scoped
   cleanup. A discarded actual `unsubscribed` reply reconciles to actual
   `notSubscribed` using one durable operation. Different/unknown cleanup identity
   withdraws local observation and retains native uncertainty without dispatching
   an unqualified native effect. Active thread IDs are bounded at 1,024; retired
   IDs are removed rather than accumulated. New creation notifications stay
   provisional until the actual returned thread ID qualifies them. The original
   1,024-frame queue cap remains. Actual 1,030 retired A notifications no longer
   abort B RPC; all three active B notifications survive once, with no foreign
   actor activity or raw tool-response authority exposed.

## Contemporaneous evidence and honest chronology

`raw/chronology.json` records exact commands, observed tool-return wall times,
exit codes, source inventories/archives and classification. Every genuine RED
archive precedes its product fix. `raw/native-oracles.json` indexes actual IDs,
cursors, stop receipts, received frames and cleanup journal observations. Full
raw logs retain the public registry, both component identities, exact native
profile/config/binary/schema pins, and identity-fenced namespace cleanup.

- Initial receipt: `red-initial.log` genuinely failed before adoption repair;
  `green-initial.log` failed because the fixture read an intermediate projection
  before history existed. `green-initial-corrected.log` passed after correcting
  only the public readiness predicate, with the failed run retained unchanged.
- Pagination: initial `red-pagination`, `red-pagination-corrected` and
  `red-pagination-native-timeout` are uncredited setup failures (query/pause
  deadlock, then unacknowledged local/read-only setup). The explicit own-environment
  `red-pagination-remote-setup.log` is genuine RED: 202 actual native setup IDs,
  exact target on page2, page1-only product rejection. Its later oracle lines
  were initially missed in a compact status scan; root was corrected promptly.
  An already-launched persistent-setup trace also reproduced genuine RED and is
  retained under its true diagnostic purpose. Final GREEN uses the proven
  ephemeral own-environment setup; no extra model turns or setup retries are
  presented as native success.
- Retirement: serial setup exceeded the fixed 15-second setup bound before
  producing the full reservoir and is uncredited. Batches of at most64 actual
  metadata calls produced 1,033 acknowledgements/1,030 A notifications in1.123s,
  preserving that deadline. Genuine `red-retired-events-batched.log` received
  1,011 A name frames plus existing activity before the unchanged1,024-frame
  cap aborted B RPC; the old connection was discarded and all B name frames
  lost. `green-retired-events.log` passed with real lost-ack reconciliation.
- Combined covering command passed93.02s: three new registry native regressions
  and two inherited process regressions matched by `repair2_`. Three declared
  cursor transport negatives, fmt, strict clippy and diff-check also passed.
  No separate preimplementation RED is claimed for supplemental transport
  coverage of the already-green cursor traversal slice.

## Exact-source gate and resource disposition

Frozen full gate is terminal PASS: 64 tests (4 transport units, 21 native
registry tests, 4 process fixtures, 20 public controller tests, 15 runtime tests),
test command762.44s, fmt0, strict clippy0 and sequential normal offline build0.
`raw/gate-commands.json` retains exact commands/exits/UTC starts and unchanged
source verification. Root independently reported the same64 tests plus all checks
PASS on6f55, but those independent reports remain root-owned.

Read-only final author resource audit (`raw/gate-resources.json`, reproducible with
`raw/observe_resources.py` from this tree's scratch evidence) finds
724 positively retained owned identities,
0 matching live. All260
retained whole-process namespace disposal receipts are terminal; all listed
unique native fixture roots are absent. The audit only reads retained PID/start
identities and readiness, never signals foreign/user processes. Fixture cleanup
uses retained qualified namespace pidfds. Interrupted or failed runs without
emitted identities are not claimed as a complete inventory. Final source matches
all25 pre-command file hashes. Prior R1 evidence has no diff from the exact base. Pre-command complete
25-file inventory `raw/gate-source.json` SHA256
`aa14290ed730fd234f23d32f941772a725b982203574ba428e1d6f7dd8063d91`;
archive `raw/gate-source.tar.gz` SHA256
`362778b0184a29fd70dd8f0a023208f68817f7722c47221a72adbec428ae5e72`.
The source remains unchanged while testing; normal offline build follows tests
sequentially to preserve fault-injection executables during native fixtures.

## Limits preserved

Linux pidfd/process identity and qualified supervisor ownership remain mandatory;
supervisor restart with surviving ambiguous workers stays unknown with capacity
held. The test-only barriers/exit/ack-loss faults cannot be enabled in the normal
build. Native RPC errors, malformed/over-budget scans and unresolved cleanup
retain explicit uncertainty. The proof is bounded deterministic protocol/process
qualification; it does not establish every possible listener, store, actor-event
or live-provider failure. Controller-only app-server authority remains outside
all tool realms; native raw server-wide authentication is not UI/tool authority.
Own executor transient bearer scope and prior isolation qualifications are
unchanged. PTY attachment, routed questions/dynamic tools and live actor/provider
qualification remain gated by downstream tickets05/06. No accepted02 or live
baseline claim is made here.

## Independent review disposition — not accepted

While the frozen gate ran, parent root reported scoped Standards PASS and
branch641 FAIL with two new Medium recovery findings. Spec is terminal FAIL, independently confirming exactly the same two findings
and no further important delta findings. Root retains those formal reports. Root retains the independent report at
`.scratch/sqlite-controller/reviews/02/registry/repair2/branch-641.md` in its
coordination checkout; it is not rewritten or claimed as author evidence here.

- Confirmed `thread/start` receipt adoption from a null binding thread ID does
  not yet add the adopted exact thread to the receive allowlist before the
  recovered first turn, so actual initial notifications can be discarded.
- Cleanup retries can repeat history reads and overwrite a saved terminal
  shutdown receipt with unknown; subsequent revocation can leave a pending
  unsubscribe unreconciled.

These findings remain open. Passing covering/full-gate evidence does not
supersede the review. Root ordered source frozen through clean R2 evidence-only
delivery and will dispatch the consolidated final registry repair3/3 afterward.
No source repair for these findings is included or retrospectively claimed.

Exact fresh source gate, from the standalone tree (normal build is last):

```sh
cargo test --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml --features fault-injection -- --nocapture --test-threads=2
cargo fmt --manifest-path crates/sqlite-controller/Cargo.toml --check
cargo clippy --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml --all-targets --features fault-injection -- -D warnings
cargo build --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml
```

To replay the retained author read-only resource inventory in a new owned tree,
copy this handoff's `raw/*.log` into that tree's ignored
`.scratch/sqlite-controller/02/registry/repair2/`, then run the retained
`raw/observe_resources.py` from that tree. It writes the observation JSON only in
that ignored scratch directory. Root's independently generated gate/resources
and review reports are retained separately in canonical
`evidence/02/registry-independent-review/repair2`; author does not rewrite them.

Source/fmt/strict checks pass. The staged verbatim raw-log diff check exits2 for
original trailing whitespace/blank output lines (`raw/evidence-raw-diff.log`).
Those bytes are preserved as evidence; the scoped handoff/manifest diff check
passes. This is an evidence-format result, not an omitted product gate failure.
