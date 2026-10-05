# Ticket04 implementation handoff

Implementation source: `2b1887b8c8c6e0ea806487271ae77fc3b743f63d` on
`sqlite04/git`, based on `d121795f7c15c02a2c2cec685f5bc30fb570a363`.
This commit contains product code, public fixtures, documentation and the bounded
standalone gate. The later evidence commit changes only this evidence directory.
No source integration, shared ref mutation, production runtime or paid model run
was performed. Independent Standards/Spec/branch review and acceptance remain
root-owned. Full02/04 acceptance and live release are not inferred here.

The implementation provides complete private Git repositories via the actual
supervisor's `TrustedWorkspacePreparer`, including native Codex executor profiles.
Capture, separate fixed-suite OS-isolated verification, authorized acceptance and
physical protected-ref landing have separate durable identities. Protected Git
ref effects serialize, journal before execution, require FF ancestry and expected
base CAS, and recover lost acknowledgements without duplicate effects. Retained
lineage explains later refs; unexplained divergence blocks dispatch. Inventory and
cleanup preserve dirty/untracked/live/unknown/shared work and require observed
termination, retained bytes, exact ownership and explicit local disposable refs.
The common backend supports Git; jj remains unqualified.

`crates/sqlite-controller/README.md` documents the trusted policy and API. The
controller and executor supervisor load the same `CORENIUS_GIT_CONFIG` policy.
The public capture command accepts only report content and derives its workspace
from the current attempt grant. Verifier suites, tools, environment and arguments
are trusted policy inputs, not worker request fields. `git_workspace.rs` exposes
`create_workspace`, `capture_candidate`, `materialize_candidate`, `compare`,
`land`, and `retire_workspace`; `git.rs` owns controller authority and journals.

## Frozen verification

All rows below ran on the exact source commit above. Complete 34-file manifests,
source/test/gate archives and SHA-256 digests are in `source/`; actual before/after
file digests match for every frozen row. `source-pairs.json` records pair status.
All Cargo invocations use the standalone manifest, locked/offline dependencies,
and bounded `--test-threads=2`.

| Log under `.scratch/sqlite-controller/04/` | Actual result |
| --- | --- |
| `frozen-affected.log` | Git 2/2 (61.89s); Kata 6/6; process 4/4; public 20/20; runtime 15/15: 47 passed |
| `frozen-native.log` | Both actual native Git and pre-unsubscribe restart topologies: 2/2 (35.13s) |
| `frozen-pending-controls.log` | Three inherited cleanup/pending controls, each both topologies: 3/3 (122.95s) |
| `frozen-default.log` | Default-feature library 4, Codex 6, Git 1, process 3, public 20: 34 passed; empty suites/doc tests also passed |
| `frozen-fmt.log` | `cargo fmt --check` passed |
| `frozen-clippy.log` | All-target fault-injection strict Clippy passed |
| `hostile-parent.log` | Actual parent mountpoint replacement attempts refused; final-file symlink/outside read denied |
| `frozen-build-last.log` | Normal-feature build passed last |

Exact reproducible scoped gate:

```sh
cargo test --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml --features fault-injection --test git --test public --test process --test runtime --test kata -- --nocapture --test-threads=2
cargo test --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml --features fault-injection --test codex ticket04_ -- --nocapture --test-threads=2
cargo test --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml --features fault-injection --test codex repair3_ -- --nocapture --test-threads=2 --skip repair3_adopted_thread_delivers_received_first_turn_notifications
cargo test --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml -- --nocapture --test-threads=2
cargo fmt --manifest-path crates/sqlite-controller/Cargo.toml --check
cargo clippy --manifest-path crates/sqlite-controller/Cargo.toml --locked --offline --all-targets --features fault-injection -- -D warnings
python3 docs/efforts/sqlite-controller/evidence/04/hostile-parent.py crates/sqlite-controller/target/debug/corenius-sqlite-controller
cargo build --manifest-path crates/sqlite-controller/Cargo.toml --locked --offline
```

The full standalone FI gate remains available in
`tools/agents/checks.d/70-sqlite-controller.sh`; it was updated to two test threads.
Unchanged FI native25 were not routinely rerun. No full-FI aggregate result is
claimed by the scoped gate above.

## Public failures and retained corrections

The initial public inventory test failed because the query was unsupported
(`git-seam-red.log`). The first native shared/separate pre-unsubscribe restart
proof genuinely failed 2/2 in 45.01s (`native-restart-red.log`): retained terminal
stop evidence referred to the old incarnation, so no first unsubscribe could
qualify after restart. Minimal correction selects the fresh, current, qualified
terminal cleanup observation only if no unsubscribe operation exists. It keeps
the original terminal receipt immutable. First GREEN failed 2/2 in 30.45s
(`native-restart-green.log`) because the test used a stale registry executor
execution document as its final resource oracle. The corrected oracle reads the
actual attempt snapshot and qualified executor namespace pidfd. GREEN2 passed
2/2 in 30.93s (`native-restart-green2.log`). These are distinct retained runs.

`git-protocol-second.log` retains the actual first Git response-deadline failure:
C and D were captured, but verification exceeded the transport's old shared
one-second total deadline. Request read and response write now have separate
budgets; bounded concurrent serving permits cancellation during capture while
SQLite still has one transaction writer. `cleanup-lost-ack-red.log` retains the
actual after-removal crash failure: retry reopened a missing workspace. The fix
requires the same pending operation, retained archive/owner/original binding and
qualified stop before confirming absence. Both before/after cleanup recovery
modes passed in the frozen public driver.

`capability-red.log` retains the supported-cleanup projection failure. Current
Kata/context projections now report mechanism qualification and retain individual
observed dispositions. `affected-gates.log` retains a subsequent 5/6 Kata test
failure caused by assuming both admitted attempts were running; the actual second
attempt was starting. The corrected oracle preserves the exact initial observed
state tuples across replacement context. Historical summaries were not rewritten,
and mechanism support never establishes resource termination.

Retained uncredited setup/oracle failures include the first worker's absent hooks
directory with empty Git templates (`git-protocol-first.log`), a wrong snapshot
base key (`git-protocol-third.log`), valid revision conflict during concurrent
admission (`combined-fourth.log`), a missing test barrier (`cleanup-cancel-red.log`),
initial hostile mutation not actually executing its filter (`focused-git-green.log`),
and a malformed nested worker string (`bounded-output-red.log`). They are not
credited as product RED or complete GREEN. Early live resource disposition was
independently checked in `observations/first-oracle-failure-resource-check.json`.

Older exploratory RED/GREEN records have complete contemporaneous **before**
source/test archives, but no contemporaneous after capture. That limitation is
explicit in `source-pairs.json`; no retrospective after snapshot is fabricated.
Current frozen proofs and newer native-Git/output-bound/assembled proofs have
matching actual before/after manifests.

## Actual Git, native and hostile outcomes

The frozen Git fixture executes nine real modes: normal B/C/D/D2, both ref crash
boundaries, divergence, hostile capture, excessive verifier output, capture/cancel
race and both cleanup crash boundaries. Two workers run concurrently at B and
produce independent C/D repositories and candidates. Their separate fixed
contracts pass. C lands, D is refused stale, a new private attempt starts on C,
D2 preserves C and supplies the combined change, old proof is refused, new
combined verification/acceptance lands D2 by serialized FF. D and its original
proof remain retained. Exact commits, trees, candidates, suites, environments,
reports, refs, journals and cleanup outcomes are in `observations.json` and the
actual fixture DBs/artifacts in `fixtures/`.

Worker-owned fsmonitor/upload-pack probes really execute inside the owned worker
boundary and fail their attempted outside writes. The clean-filter capture probe
really executes, creates a final-file `/scratch/candidate.bundle` symlink, and
fails an outside read of a unique owned sentinel. Capture refuses, the sentinel
is unchanged, and its bytes/digest are absent from retained candidate artifacts
and responses. Capture bundle scratch is now private tmpfs; bounded stdout carries
the validated regular-descriptor bundle instead of a host read of writable
scratch paths. The fixture's additional `/scratch/ancestor` symlink is **not on the
capture pathname**, so that second symlink is not an actual parent-replacement
oracle. No RED exploiting the earlier host scratch read was credited. The evidence-only
`hostile-parent.py` subsequently runs unchanged frozen product through the public
command and actually attempts renaming `/scratch` (errno16 EBUSY) and replacing it
with a symlink (errno17 EEXIST). The parent dev/inode stays identical, a final-file
symlink is created, outside read is denied, and bytes/digest/owned host pathname
are absent from result/artifacts. `frozen-hostile-parent-before/after` source
manifests match; `hostile-parent.log` and its fixture archive retain the outcome.
Host stdout/stderr reads are bounded before allocation; the actual excessive
verifier-output mode fails verification and acceptance.

The two native proofs pin Codex 0.159.3, its code-mode companion, catalog/config,
launcher and exact native schema. Real shell/apply_patch tool execution occurs
under the actual supervisor's native profiles. Shell commits in private Git are
captured and independently verified before native cancellation/restart. The app
process is killed through its positively owned pidfd/start/namespace identity;
the replacement retains store ID and changes incarnation. A first unsubscribe
uses fresh terminal qualification, while the original receipt stays byte-for-byte
identical. Actual executor snapshot exit and pidfd readiness establish stop.
Inherited controls retain the same pending unsubscribe ID and original operation
fences; controller-connection or native-incarnation mismatch remains unresolved.
Original disposed `list_turns` refusal and prior uncertain observations remain
historical evidence; no production native issue is inferred or rewritten.

## Retention and limits

`pins.json` records source/base, Cargo.lock, Rust/Cargo, actual Git/Python/bwrap,
pinned Codex/companion/schema and Kata API0.24 source/binary identities.
`build-pins.json` retains compiled executable digests and Cargo feature metadata.
`test-results.json` retains terminal summaries; `observations.json` indexes actual
JSON outcomes/process records; `retention.json` gives paths/byte counts/digests for
compressed raw logs and owned fixture archives. `record.py` is the evidence-copy
reproducer. All original ignored raw logs/archives remain untouched in `.scratch`;
tracked compressed copies make them available across clean checkouts. Fixture
archives contain untrusted Git metadata/symlinks as evidence; they were never
host-executed. Do not host-execute retained worker Git configuration.

Only owned local fixture resources were signalled. Git fixture records retain
PID/start/pidfd/namespace bindings and whole-process disposition, including early
setup failures. Native process/namespace records retain exact identities and
fixture cleanup results. Unknown application cleanup disposition is distinct from
actual process termination. Dirty/shared/user working trees and historical
branches/evidence remain preserved.

The read-only `/usr` toolchain is pinned at selected executables and fixed boundary
configuration, not a hermetic dependency image. Live model/task qualification,
PTY/gateways, jj, full assembled backup/restore and source integration remain
outside this implementation. Root reviews must adjudicate the linked642 proof
and capture-boundary evidence before full02/04 acceptance or destructive live
cleanup/retirement/replacement-lead release.
