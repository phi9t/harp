# Ticket26 repair1 candidate

Fixed base `720ab7783e1b36e9e8a675f497f9c66ef2b1d87b`, branch
`codex-intent26/contracts`. Round0 was rejected by both independent review axes.
The retained decision admits four consumer repairs; this candidate is ready for
new root-owned review, not ticket acceptance. No commit, staging, branch rewrite,
remote operation or tracker change was performed.

The four confirmed paths now have real behavioral RED followed by GREEN:

| Finding | Genuine RED observation | Implemented boundary and positive control |
| --- | --- | --- |
| Fresh EpisodeGateway operation after suspension | HTTP prerequisite amendment followed by actual public `prepare` returned a new Prepared Start for a previously Ready Worker execution | Dependency fence runs after cached receipt lookup and before new insertion. Earlier persisted Start receipt remains identical, inspectable and reconcilable after suspension and cancellation. |
| Legacy mapped acceptance | Legacy Lead Accept returned HTTP200 and `accepted:true` for mapped work with a genuinely measured v1 report but no admitted v2 proof/outcome | Fresh legacy Accept requires unmapped work. Both Lead and Owner mapped attempts refuse409. Unmapped measured v1 acceptance and exact historical cached receipt survive later mapping; no v2 outcome is written and the old measured Episode cannot rebind. |
| Delegated prerequisite removal | Current explicitly delegated Lead amendment returned200; durable prerequisites became empty and a fresh dependent Attempt actually returned200 | Delegated Intent amendments preserve current exact prerequisite projections, including explicit revalidation. Refusal leaves suspension intact. Explicit Owner amendment succeeds and its fresh grant succeeds. Existing exact proof revalidation controls remain passing. |
| Cancelled v2 results and replay | Three independent cases returned200 after real CancelAttempt: fresh research proof, fresh acceptance, exact acceptance replay | Existing cancellation fence runs for v2 outcome proposals/bindings/proofs/acceptance before cached lookup. All three refuse409. Reads, evidence references, historical outcomes, original receipts and capacity/admission counters are preserved. |

Changes are confined to the current consumers, focused contract helpers, public
tests and explanatory README text. The nonblocking Data Clumps suggestion was
not changed. The immutable migration and v1 candidate/report formats remain
unchanged.

## Executed proof and attribution

`runs.jsonl` retains all eleven bounded attempts with exact argv, environment,
source hashes, process terminal status and raw `.log.txt` hashes. Cargo used the
existing PostgreSQL runner with `--offline`, its internal `--locked`, private
target, two build jobs and one test thread. Each invocation has a 1800-second
bound. No Temporal runner, full gate, provider, Trae or new physical-runtime probe
was used. The existing supervised verifier fixture supplied genuine monitor
measurements for acceptance tests. EpisodeGateway preparation uses its existing
public Rust API against the same real PostgreSQL fixture as raw HTTP admission
and supersession; no new HTTP route or mock was introduced.

Two initial Episode test setup failures are retained separately: a Kernel method
was initially called on Supervisor, then the fixture attempted two active
Episodes for one principal. Neither is labeled behavioral RED. The genuine
Episode RED is `red-episode-start-after-suspension-3.log.txt`. The other relied-on
REDs are `red-mapped-legacy-accept.log.txt`,
`red-delegated-prerequisite-removal.log.txt` and
`red-cancelled-contract-results.log.txt`. The cancellation control was corrected
to respect the existing read boundary: unpromoted proposals never grant legacy
artifact HTTP read scope. Immutable investigation retention is checked through
the actual ArtifactService Owner collection and store read, alongside successful
authenticated snapshots.

The final focused suite passed 15/15. The final exact-source compatibility run
`green-final-repair1-compatibility.log.txt` passed 81/81: library 9, artifacts 11,
gateway 4, contracts 15, kernel 16, mailbox 1, mailbox artifacts 2, delivery 3 and
supervisor 20. Every source hash captured by that final runner matches the frozen
worktree. `run-source-deltas.json` identifies every older-run difference from the
final source; round0 runs are historical evidence, not proof for repaired bytes.
`public-records.jsonl` is only extraction metadata with raw log line hashes.

## Cleanup, retained artifacts and freeze

Process completion, PostgreSQL runner cleanup, physical verification completion
and `Fixture.finish` are distinct facts. The runner returns after its existing
EXIT cleanup trap. The genuine Episode RED and mapped acceptance RED unwind
before `Fixture.finish`; their original owned temporary files were retained,
not silently discarded. `retained-red1-fixture.json` archives 59 file records for
the unlaunched Ready allocation. `retained-red2-fixture.json` archives 72 file
records, including candidate bundle, exact workspace/provenance and monitor
bytes. Its actual monitor result records exit 0 and namespace stopped before the
failed Accept assertion. No new PID/namespace probe substitutes for that result.
Original fixture paths remain recorded and retained. Delegation/cancellation RED
tests finish their fixtures before failing assertions. Passing contract tests
perform their explicit workspace stop/release and fixture cleanup. Exit101 alone
is not claimed as proof of complete fixture cleanup.

`round0-archive-map.json` preserves every prior source/evidence byte and the exact
decision/review inputs. Top-level round0 author records were not refreshed or
relabeled. `source-manifest.json` freezes 18 final source files, verified base
preimages, round0 preimages and the eight repaired-file deltas. `source.patch`
reconstructs all 18 files from the fixed base; `repair.patch` reconstructs the
repair from round0. `checks.json` records patch/hash/format and ordinary Git
selection audits. `author-freeze.json` seals all selected changes except itself;
its hash is supplied in the root handoff. Every durable archive/index/snapshot
is normally Git-selectable; scratch, cache, target and ignored runtime origins
are excluded as proof paths.

The shared Cargo/PostgreSQL slot was released after final terminal GREEN and the
runner cleanup. No further resource run follows the freeze. Root retains new
independent Standards/Spec review, branch verification, commit and integration.
Ticket26 remains claimed. Scope allocation, Kata qualification, Plan execution,
live lead demonstration and assembled v0.2 recovery remain excluded. This packet
does not claim whole-v2 acceptance; no implementation authority question remains.
