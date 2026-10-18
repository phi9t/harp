# Ticket04 repair1 implementation handoff

Product SOURCE: `e6ab0abb9ff58e08669398034e693a2190ef46a1`, full tree
`d2df74745f883262d81e53b83cab177328e87408`, branch `sqlite04/git`.
Its parent is retained initial evidence tip `f85fcd15a912a59a9af7e4d0da56329f38a5c298`;
initial product source `2b1887b8c8c6e0ea806487271ae77fc3b743f63d` and all its
historical evidence remain unchanged. This is consolidated repair1/3 for nine
initial review findings. Root owns independent reviews, acceptance and source
integration. No integration, shared branch rewrite, production/user runtime,
paid model call, or fourth ticket02 repair was performed. Full02/04 acceptance,
retirement/replacement, destructive cleanup and live release remain gated.

The eight changed product/test files repair the approved existing Git baseline:

1. Trusted inspection uses Python `-I` within its existing OS boundary. Capture
   manifests and inventory retain the actual full command, cleared environment,
   executable pins and owned process evidence. Candidate evaluation still uses
   the controller's configured fixed suite.
2. Landing revalidates requester epoch, candidate, current proof and accepted task
   state in the transaction admitting the actionable ref intent. A replacement
   before admission refuses the effect; replacement after admission preserves
   the durable authorized decision. Physical FF/CAS effects remain serialized.
3. Normal confirmation and startup recovery use one idempotent publication path
   for revision, event, receipt and Kata effects. Recovery uses retained authority,
   candidate/tree/base/ref/policy provenance and does not require the old caller.
   Later explained lineage never rewinds the current base.
4. Fresh capture/accept/land cannot reopen accepted/landed lifecycle state.
   Recorded stable-key replay still returns its retained prior result.
5. A newly admitted native unsubscribe stores its selected stop qualification.
   Retries reuse that exact qualification and original logical ID, connection,
   store and incarnation; legacy intents preserve their original shape and fence.
6. Inventory takes a short coherent database snapshot and releases its connection
   before untrusted Git observations. Revision/age/drift distinguish that snapshot
   from current controller state; the physical Git gate still protects cleanup.
7. Pipe readers bound stdout/stderr during production, and final limit checks
   reject oversized output even when the verifier exits zero. Owned child pidfd,
   kernel start, exit/signal, wait and configured budgets are retained.
8. Tar streams through the same bounded runner: 32MiB archive, 1MiB diagnostics,
   30-second deadline. No expanding host archive file is created. Failure retains
   bounded producer diagnostics and preserves the attempt for retry.
9. Cleanup journals the original attempt-root identity, owner/binding, observed
   stop, archive and entry manifest before deletion. Retry accepts only unchanged
   surviving entries of that original root; missing entries can be a real partial
   deletion. Replacement/alias/new or changed work is preserved. The original
   archive is retained, rather than recaptured from a partly deleted root.

Public controller APIs remain the documented capture/verify/accept/land,
workspace inventory and explicit owned-ref retirement commands. Internal cleanup
now uses `cleanup_manifest` and `retire_retained`; setup remains exclusively
new full repositories through `TrustedWorkspacePreparer`. Git is required;
jj07 remains unqualified. Worker metadata is never host-executed.

## Public RED/GREEN and correction chronology

All logs below are retained under `raw/`; original logs are in
`.scratch/sqlite-controller/04/repair1/`. Every credited run has complete
34-file source/test/driver/gate manifests and archives captured before and after.
`source-pairs.json`, `observations.json`, `retention.json` and `test-results.json`
record exact identities and outcomes. Failed setup/oracle runs are retained and
are not credited as product proof.

| Finding | Genuine public failure | Corrected actual proof |
| --- | --- | --- |
| 1 imports | `shadow-red`: worker module actually executed, forged clean status | `shadow-green`: actual untracked work retained, shadow module did not execute |
| 2 authority | `authority-red`: revoked lead advanced protected ref | `authority-green`, `publication-green2`: before-admission refusal / after-admission durable effect |
| 3 publication | `publication-red`: ref/base landed with no landing event/revision | `publication-green2`: both actual ref crash boundaries, replacement and exactly-once receipt/event/revision |
| 4 lifecycle | `terminal-capture-red`, `terminal-accept-red`, `accepted-capture-red` | `lifecycle-green`, `publication-green2`: fresh-key capture/accept/land refusal and stable replay |
| 5 stop retry | `native-final-driver-red`: both actual topologies stalled on old stop | `native-final-driver-green`: both same-ID retries qualified and retired, 2/2 PASS in 35.75s |
| 6 inventory | `inventory-red`: actual slow clean filter blocked controls | `inventory-green`: status/progress/cancel promptly passed while inspection remained blocked, 2/2 PASS in 8.30s |
| 7 final output | `final-output-red`: actual exit-zero verifier minted PASS after oversize | `final-output-green2`: rejection, 2/2 PASS in 7.62s; pipe runner and explicit exit-zero/limit oracle pass in final controls |
| 8 producer | `archive-red`: sparse64MiB source expanded beyond32MiB before rejection | `archive-green`: bounded pipe failure preserved root; owned truncation followed by public retry succeeded |
| 9 partial delete | `partial-red`: real .git/owner deletion + crash left same operation stuck | `partial-green` and `partial-controls-green`: same operation recovery plus replacement/alias/new-work preservation |

`publication-green` retained a compile/type-inference failure before the corrected
`publication-green2` 2/2 PASS in 33.72s. `final-output-green` retained a failed partial
correction: the recorded limit was still ignored by the verdict. The corrected
verdict passed only in GREEN2. The actual RED expanded archive peak was 67,194,880 bytes for a 67,108,864-byte
sparse source. Fixed producer stream retention is bounded to 33,554,432 bytes.
Producer
limit/exit/start/pidfd-wait evidence and retry outcomes are in the corresponding
observations and complete fixtures; the fixed runner retains at most 32MiB stream
bytes, with no archive file expansion.

The first native `native-lost-ack-green` retained a shared active-peer connection
mismatch while separate passed. This is correct fail-closed evidence, not a
waiver. The final scheduling retires the peer before restart to isolate same
connection retry. `native-lost-ack-green2` then failed only when the fixture tried
to cancel that already-retired peer; this disposal/control failure is uncredited.
After correcting disposal, genuine RED was re-established under the identical
final peer-retired driver against a uniquely owned scratch source materialization
with only the retained initial `codex.rs` overlaid. No preserved checkout/ref was
reset or rewritten. `native-selection-replay.json` pins that recipe and exact
Rust/Python driver identities. The matched RED failed 2/2 in 50.94s and GREEN
passed 2/2 in 35.75s. Their observations explicitly retain the immutable original
stop, selected current stop, original connection/store/incarnation and same
logical unsubscribe ID; selected stop is reused after actual discarded native
unsubscribe/history acknowledgements. Fresh source-frozen native and inherited
pending controls are separate below. Historical failed observations remain.

Supported Kata proof `kata-publication-green2` passed 2/2 in 9.47s against the real
pinned API0.24 daemon: both physical ref crash boundaries, replaced lead, exact
recovered landing event/revision, confirmed metadata+comment effects, cursor,
and repeated recovery/reconciliation without duplicate effects. First
`kata-publication-green` failed its revision assertion because autonomous Kata
acknowledgements added further events. The corrected fixture pauses autonomous
runtime for the crash-domain observation and then invokes the supported public
Kata reconcile API for actual metadata/comments. This case establishes explicit
reconciliation, not autonomous synchronization. Existing03 autonomous proof is
inherited. Fresh issues and supported `create_kata_task` source snapshots produce
actual WorkRefs; the full configuration, API trace, SQLite journal, fixed suite,
Git policy and effect/readback documents are retained in those fixtures.

Mechanism qualification remains distinct from each resource disposition. Native
pending connection/incarnation mismatch controls remain unresolved, retain
capacity and refuse new input. Current qualified cleanup support never marks a
running/unknown resource stopped. Original branch642 failures and disposed
`list_turns` uncertainty remain historical evidence; no native production issue
is inferred. Acceptance of linked642 and ticket04 still belongs to root reviews.

## Coordinator checkpoint matrix

`nine-finding-matrix.json` explicitly records all nine original failures, declared
hypotheses, fixture changes and their rationale, exact paired source identities,
focused versus independent proof, retained unresolved controls, and the next
discriminating independent check. It is the compact evidence checkpoint for the
owner monitoring rule; private reasoning is excluded. Frozen SOURCE remains
unchanged. The next checks are root's already-required independent frozen gate
and re-reviews, rather than an additional author scope or repair round.

## Frozen gates

All following gates run on SOURCE above, with unchanged complete paired source
manifests. Cargo uses the standalone lockfile offline and two test threads.
`gate-*-run.json` contains exact commands, source identity, exit, timestamps and
elapsed time. The normal-feature build is last. The root initial 78-test gate
belongs to initial source 2b1887b and is not repair1 proof.

| Frozen log | Actual result |
| --- | --- |
| `gate-affected.log` | Git 2, Kata 8, process 4, public 20, runtime 15: 49 passed; 163.01s |
| `gate-native.log` | Four actual ticket04 shared/separate first-cleanup and lost-ack native cases: 4/4; 72.05s |
| `gate-pending-controls.log` | Three inherited pending/terminal/connection/incarnation controls, both topologies: 3/3; 123.23s |
| `gate-default.log` | Default library 4, Codex 6, Git 1, process 3, public 20: 34 passed; 176.60s |
| `gate-fmt.log` | Formatting check passed |
| `gate-clippy.log` | All-target FI strict Clippy passed |
| `gate-build-last.log` | Normal-feature build passed last; 3.30s |

The scoped gate runs changed/current suites and the inherited pending fences;
unchanged native25 are not routinely repeated by the author. Root independently
owns its complete frozen gate and both review axes/branch checks. At this
checkpoint root reports scoped Standards/Spec repair PASS, branch646 FAIL2, and
the complete root gate still running. Branch646 source-confirms fresh verification
after acceptance can invalidate the proof while new acceptance is refused. Its
legacy pending-journal compatibility claim is under root adjudication. Acceptance
remains withheld; SOURCE is unchanged and no migration/product patch is made
before the consolidated authorized repair2 brief. Review hashes are retained in
`repair1-review-pins.json`; this handoff does not overrule those reviews.

Reproduce through `run.py` or the exact commands in the run records. The complete
standalone FI gate remains `tools/agents/checks.d/70-sqlite-controller.sh`.
`frozen-source.json` pins all source files; `tool-pins.json` pins the compiler,
Git/Python/bwrap/tar, Codex0.159.3 and Kata binary/source/API. `input-pins.json`
pins the canonical brief/spec and finalized initial reports. Complete raw logs,
archives and immutable before/after artifacts are retained; evidence tip adds
only this repair directory and does not change SOURCE bytes.
