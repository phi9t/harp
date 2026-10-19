# Ticket03 consolidated repair1/3

Clean frozen source commit:
`4d5fbff0323f27c6dcb30aec5cf3df7a839511e0`, from evidence tip
`40d4805ce34d74521f3473b7f8ad2b9f7451d0c2` (product `d201642`).
This is the same exclusive author/tree/branch and authorized repair round.
Root owns scoped independent conformance and the final gate; neither acceptance
nor release is claimed. Product files remain unchanged while retaining evidence.

## Six verified Kata repairs

| Finding | Public RED | Minimal correction and covering GREEN |
| --- | --- | --- |
| Snapshot pathname check/read race | `repair1-final-driver-snapshot-red`: actual outside bytes captured after a declared post-check FI barrier and real path replacement | Open each directory relative to a retained descriptor with `O_NOFOLLOW`; validate/read the same opened regular file. Final-file substitution refuses; ancestor substitution reads only the originally opened registered directory. Both public controls pass and outside bytes never appear in participant context. |
| Autonomous reconciliation starves socket/control | `repair1-responsiveness-red2`: genuine upstream responses delayed300ms each cause control receipt to exceed800ms | A controller-owned thread performs autonomous Kata reconciliation, retaining short SQL transactions and the serialized effect gate. The same real delayed-response driver gets its pause receipt below800ms. Explicit diagnostic reconciliation remains synchronous. |
| Task A failure blocks task B publication | `repair1-final-driver-isolation-red`: A namespace drift withholds all B publication | Select only each task's earliest pending effect, rotate attempted heads behind untouched heads, and withhold later effects only for that failed task. A remains unknown with later A effects prepared; healthy B metadata/comments confirm. |
| Valid maximum progress becomes unpublishable | `repair1-final-driver-progress-red`: accepted65536-byte progress creates unknown comment | Metadata uses a4096-byte UTF-8 excerpt and reports original length/truncation; visible comment framing is bounded to8192bytes plus the stable operation marker. Full local progress is unchanged in the event ledger. Real65536-byte ASCII and escaping/multibyte controls publish and retain exact local text. |
| Deterministic fixture runs without its feature | `repair1-default-feature-red`: actual real fixture loses its expected unknown acknowledgement state under autonomous reconciliation | Gate the complete Kata test module on `fault-injection`. Feature-off command passes with0fixture tests; the feature-on suite runs the inherited protocol and five explicit regression tests. |
| Unchanged source recovery retains obsolete outage | `repair1-final-driver-availability-red`: real daemon restart with identical source and no effects leaves old outage drift | Compare availability/projection drift independently of source equality. Successful unchanged observation retains a recovery event, clears obsolete availability uncertainty, preserves failure events, and restores independently real external projection drift. Immutable admitted/current identities remain unchanged. |

All focused Kata boundary runs use the exact real pinned Kata binary/API/source
from the initial report. Every credited RED/GREEN has a contemporaneous complete
29-file manifest and source archive in `source/`, plus raw logs in `logs/`.
The final identical Python driver SHA256 is
`18b6d3a5698d3690547157ff5e1d983871bb55d8c4f31aefeabdeb0ddbc1299e`.
The snapshot RED source adds only a declared FI barrier to the old pathname
behavior; its purpose and precise source variant are retained, not called an
uninstrumented race. It fabricates no Kata reply. The first responsiveness
attempt passed because its startup probe waited out the delay; it is retained
as an inadequate oracle, not credited RED. RED2 starts from actual socket
readiness before asking for the control receipt. The other four REDs were
repeated with that corrected identical driver before implementing fixes.

## Bounded inherited native diagnosis and fixture correction

The first unchanged-source inherited test passes1/1 in58.59s across both actual
topologies; `repair1-native-unchanged-before` pins its exact source. That rerun
does not resolve the root full-gate failure. A disposable diagnostic variant
adds state prints and post-cleanup archives with unchanged assertions/deadlines.
Its first archive helper had a `Path` NameError, retained separately as a setup
failure. Diagnostic2 passes both topologies and archives the complete fixtures.
Its samples prove readiness during global hold, not readiness before that hold.

Root independently adjudicated a source-proven scheduling trap: A's first
recovered FI snapshot set global hold, but B could still need a later retry;
later drains returned early while the fixture waited for both actors. The
original root failure separately records B's actual `list_turns is not supported
yet` response and lacks final hold/readiness diagnostics. Root authorized the
bounded correction in [the exact amended brief](reviews/brief-fi-native-route.md)
and retained [adjudication](reviews/native-fixture-adjudication.md).

The controlled scheduling RED first pins the exact admitted A/B identities under
a setup-only hold, then releases setup. A undergoes the real post-confirmation
crash/restart and real first-turn shell/patch/history/received-frame completion.
An adoption-only FI hook defers exact B before native binding RPCs until A's
snapshot, defers once again in that pass, then disarms. This is a declared
scheduling fault, not a fabricated native response or a simulation of the
original unsupported method.

`repair1-native-deferral-red` fails in both actual shared/separate topologies
inside the original30-second readiness deadline. Its captured prerequisites
are true: immutable A snapshot equals its saved document; one thread/start and
one turn/start; exact client-message history; real shell/patch; received
active/idle frames delivered exactly once; global hold true; exact B gate
disarmed; B nonready and never retried after disarm. The source is unchanged
across command; pre-command controller/supervisor/gate/driver hashes are retained.
The separate validation JSON checks both prerequisite sets and exact A client
and operation counts.

The correction is entirely `cfg(fault-injection)` and the active adoption fixture
mode in `codex.rs`. Each exact actor's first qualified saved projection freezes
only that actor's later fixture reconciliation. B keeps ordinary retries until
its actual thread/turn/environment/UI readiness and first creation-event
projection are captured. Global hold is set only after both snapshots exist.
There is no callback wait, timeout inflation, native production change or642
cleanup repair. The original driver and all original identity/frame/client/count/
peer-creation assertions are identical between controlled RED and GREEN.

`repair1-native-deferral-green` passes1/1 in61.20s for both actual topologies.
Archives additionally confirm exact B disarm followed by a real later retry,
qualified B identity and its actual `thread/started` positive control. A's
snapshot remains unchanged. These runs establish fixture scheduling recovery;
they do not erase the root unsupported-method response or conclusively attribute
every aspect of that disposed failure. Persistent native capability failure is
not waived. Root subsequently reported its fresh scoped50/default/fmt/strictClippy/normal
build gate PASS with unchanged30tracked source/check files and31resource
identities/0matching live/7terminal disposition receipts. Independent rereviews
remain pending; full02/04 and assembled06 release
requirements remain pending.

## Terminal checks, source composition and retention

All Cargo commands use the standalone manifest with `--locked --offline`.
Feature-on tests use `--features fault-injection -- --test-threads=2`.

| Terminal check | Actual result |
| --- | --- |
| Five focused real Kata boundaries |5/5 PASS, separate retained fixtures |
| Affected Kata/public/process/runtime |45/45 PASS: Kata6, public20, process4, runtime15 |
| Library units |4/4 PASS |
| Changed exact inherited native case, both modes |1/1 PASS61.20s |
| Feature-off `cargo test --test kata` |PASS0tests, no fixture launch |
| Final post-FI fmt, strict Clippy all-targets with fault-injection, normal build |All exit0 |

The49 affected checks precede the native fixture-only correction. Exactly two
files differ from that terminal source: `src/codex.rs` (FI-only change) and
`tests/support/native_registry.py`. The exact changed native case and all final
format/lint/build/feature-off checks run on the final source. This is a composed
author50-check result, not a full final native25 rerun or a final74-test claim.
Root explicitly chose a fresh independent49+changed-native scoped gate; unchanged
24 native cases remain historical, and assembled06 requires its complete gate.

`source/repair1-frozen-source.json` pins all29 files at source commit `4d5fbff`.
Frozen archive SHA256:
`c9ce8227d838291b2207ab578c99fa44addc14213b40356e7c4997b7486a6727`.
Those hashes equal controlled-native GREEN pre/post and final post-FI checks
pre/post, and were rechecked against current files. Source was committed clean
before restoring only the author's pending evidence directory. This handoff's
follow-up commit changes evidence only.

`kata-retention.json` records22 new unique real Kata fixtures. Credentials/config
stay local600, excluded from archives; archive members were scanned for protected
Kata token bytes, and no observed owned process start identity is still live.
`fixtures/` also retains six stopped native fixture archives: diagnostic2 in two
modes and controlled RED/GREEN in two modes. Native archive hashes/snapshots and
kernel PID/start identity scans are indexed in
`source/repair1-native-resource-snapshot-validation.json`. Controlled RED retains
historically running/unknown supervisor records instead of rewriting them into
terminal success; current identity scans find0matching live executions. Owned
direct child exits and positively observed namespace cleanup receipts are in the
raw native logs. No unrelated process was signalled.

All original indexed evidence bytes were checked unchanged before the requested
append to the main handoff. Raw source archives/manifests, setup/oracle failures,
prior unknown states and original commits remain retained. No helper agents,
other checkout writes, shared-ref rewrites, pushes/landing, production tasks or
credentials, upstream model calls or tmux mutations occurred. The additive
schema/no-downgrade boundary and operator-selected reference-closure requirement
remain unchanged. Root owns further review, gate and acceptance.
