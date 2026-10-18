# Controller03 salvage repair 2: source and verification record

Commit read: `c213c8c8ec63df73ea5c0341438e4f45a33199ae`. Fixed code reviewed and tested: `ded40db69367c94f70848cae0d56b4350d9877c2`.
Owned tree: `/data02/home/philip.yang/devx/tmp/corenius-controller03-salvage-08b8c914`.
This record concerns deterministic evidence repair and the stable Kata default. It does not claim a live trial.

## Source construction and bounded repair

The five bench files were three-way merged from current main source, the retired landing seed, and the frozen controller03
review snapshot. Both retained source directories remained read-only. Only the `results.py` constants block conflicted:
readiness constants and APIs were retained alongside controller v2 and legacy v1. Four files merged cleanly. Git's merge
output normalized the controller file's final blank line; original bytes were not changed. The old results module was not
copied wholesale. Production Rust and Cargo sources match main.

Success now derives effect disposition and autonomous eligibility from observed owned service and Attempt exits. A controlled
uncertain service observation runs through the real partial and complete result paths. The existing hanging-process cleanup
fixture retains uncertainty after bounded kill without inferring an exit. Manual pause before verification retains UNKNOWN
task truth and no verifier return code, independently of orchestration FAIL. Proven cleanup resolves effects after verifier,
generic and delivery failures; observed unresolved reconciliation remains unresolved. Task FAIL stays independent of resolved
effects. This is repair 2 of the three-round budget, after the original owner's repair 1; no retry/repair authority was reset.

Explicit lead delegation extended this bounded scope when cleanup exposed retired Kata defaults. Three lookup sites now use
`Path.home() / '.local/share/corenius/pinned-tools/kata' / <approved SHA-256> / 'kata-pinned'`.
Controller/SQLite entry-point overrides use `CORENIUS_KATA_PINNED_BINARY`; the native fixture retains `KATA_PINNED_BINARY`.
Hash guards remain. The lead installed the exact approved artifact with mode `0o700`, outside disposable trees.
Receipt: `/data02/home/philip.yang/devx/tmp/corenius-salvage-a617643f/kata-stable-pin-receipt.json`, SHA-256
`c726fee4a850419a2cb65c97a7f95f89a916383696498a868978cf09aa0d37e6`.
No other configuration paths changed. Added source scope is the SQLite entry-point lookup, native fixture lookup and their
necessary loaded-module regression. All other production tool-runtime source remains unchanged.

## Exact source identities

These are SHA-256 hashes of file bytes; dashes mean outside the five-file frozen delta. All eight final hashes were pinned
before default runtime/native tests and verified unchanged afterward. All `.rs` and Cargo files match main. Under `tools/`
and native fixture support, only the two authorized lookup sites and their regression differ.

| File | Main read | Retired seed | Frozen review | Reviewed repair |
| --- | --- | --- | --- | --- |
| `bench/README.md` | `a4ef6c382f52b9247358f944b1ff4d6fe85f5cf27bc5ddeb2deec6ac18782e73` | `a4ef6c382f52b9247358f944b1ff4d6fe85f5cf27bc5ddeb2deec6ac18782e73` | `42ae4f3ba4d383f97538f93c36cbf9037c39c7a1e625cdfd397ef692b2b713d5` | `88589335ee1d24882dc059feed3c665deec3a62eab81eafe9df8ddff31ef9c6f` |
| `bench/harness/controller_mode.py` | `1cb9c37d7c07f34285013ee568939f2e4af1225a2465a0d65691908c751571a1` | `a406741e636663b4e5eb460c0f1aab0cf332ca15c0b36df59d0da48d1e2f4bf9` | `01b0f9b81d787cc29b928e925ef3130148ebdc81b0e4d7b2a4fbb02acbbb3aaa` | `88d99550dae18e8b777b043d16b89ca3b831166057344bad72c1ea5702f79372` |
| `bench/harness/results.py` | `e6376601533d5eae155c33288534770c3f85faffde760dd0af7176f7da89ebed` | `f41e4572ce1a995fa98e4524c9932a19a6f7dde0d561302780d995af13e9a649` | `3966b73e7f7c99478148ed678142a2d9f6eb3872089bade0d6ff35ee85699465` | `00062b31d699006b9b8ccd6ed7b6375910532e12451d140a2079e6ec8fd776cd` |
| `bench/harness/run.py` | `b95e0f347575e78e767c0273e29243ab18cf170943e0c881297656fbbe6a7839` | `b95e0f347575e78e767c0273e29243ab18cf170943e0c881297656fbbe6a7839` | `7d5b9afc9cf3390ed986474aa78a0265ad449ada5360b03ffdb23d9a23892944` | `7d5b9afc9cf3390ed986474aa78a0265ad449ada5360b03ffdb23d9a23892944` |
| `bench/harness/test_run.py` | `edc515e3eeee8c87d0b184f80e2eeb5d4b1e1b33ae2269c3b84678035065f48c` | `edc515e3eeee8c87d0b184f80e2eeb5d4b1e1b33ae2269c3b84678035065f48c` | `60fde4439dcf89f2171c9ec8aa98a51a1bc97a612cb0adf03ff39a7bc6506f52` | `dfddb4427c570cade4d840f84dbcf69670c5676e3ae1a7f2a4ef08ec3659cf60` |
| `tools/agents/sqlite_baseline.py` | `42d49553643c3966ef1f6a7e6e32fdc5dda445379b8ddeda29fac1e44ac8e961` | `—` | `—` | `648eb09b6df3036cfff8ee25f4cd6b638a5c7ac1b8835b88b04c5ac4f093b2f7` |
| `tools/agents/test_sqlite_baseline.py` | `b3107b8631e15c77eb20539410addc0930101595d9cdaf85a041a86396abf77e` | `—` | `—` | `d00b51756d87c3667ddaf6d03182e018840225e9f5724534a05ca42499ed39c2` |
| `crates/sqlite-controller/tests/support/kata_protocol.py` | `7afa6d2b87cc80186534c8850b16e92cc8db57b02973e7d457e0f0f0b306a7fb` | `—` | `—` | `a1b07a489391dceca4a1fb9666ac4dcff313402141796870fb34625144a085ab` |

Frozen snapshot: `/data02/home/philip.yang/devx/tmp/corenius-salvage-a617643f/controller03-frozen-review-64647782`. Seed: `/data02/home/philip.yang/devx/tmp/corenius-salvage-a617643f/retired-full-trees/corenius-landing-controller-20261007`.
Runtime red: `/data02/home/philip.yang/.cbt/cb-complete-1kdm9xx3/source-manifest.json`, source-content digest
`2ed6407ca95ab250cff4a2528a28a2bf8fb107248932a1620755d133abe41e8c`.
Red controller SHA-256: `9b60418d23e60788b85f8f443953b7b875a929ee32510707d48bf7835b4f19f0`; regression-module SHA-256:
`ea0d1a8d5c85e2e651231b58335fbe1efb61cd4fb8e22de42e8774224394a4f7`.
Pre-pin causal/mixed green source-content digest: `2b7b8473e535cae0194701d737431d56f672257a2322a4d60e136f35424c90a9`.
Green controller SHA-256: `7c6b2cb7f3d772946d6163d7653890935b554eac210fce3c67975a8d4f05d708`; regression-module SHA-256:
`dfddb4427c570cade4d840f84dbcf69670c5676e3ae1a7f2a4ef08ec3659cf60`.
The five hashes stayed fixed across the 94-test mixed suite. The later controller delta was the stable Kata lookup and
formatting of its adjacent boolean continuation, alongside the three authorized lookup/test files and README.
Final runtime source-content digest: `99cc59a970c2672f2ffb32fc91e918b0b410686719b45514ce8cd38b0a3bccd5`, retained at
`/data02/home/philip.yang/.cbt/cb-t1-zsxd_i_m/source-manifest.json`. The earlier owner's 47-test evidence was not used as a gate.

## Commands and results

Commands ran from the owned tree. Raw logs are under its `.scratch/controller03-salvage/`; preserve the full-tree archive
before retiring it so raw evidence remains recoverable by the hashes below. Initial runtime attempts stopped at the missing
retired prerequisite; the successful causal red used the separately retained hash-matched artifact.

Red: exit 1; 3 tests, 6 expected assertion/subtest failures, 95.690 seconds.

```sh
CORENIUS_KATA_PINNED_BINARY=/data02/home/philip.yang/devx/tmp/controller-bench-pinned-tools/kata-pinned TMPDIR=/data02/home/philip.yang/.cbt python3 tools/temp_root.py -- python3 -m unittest bench.harness.test_run.RunTest.test_controller_failure_disposition_follows_actual_cleanup_not_failure_stage bench.harness.test_run.RunTest.test_controller_manual_pause_before_verification_has_unknown_task_truth bench.harness.test_run.RunTest.test_controller_success_requires_observed_service_exit
```

Repair green: exit 0; 5 tests, 94.468 seconds.

```sh
CORENIUS_KATA_PINNED_BINARY=/data02/home/philip.yang/devx/tmp/controller-bench-pinned-tools/kata-pinned TMPDIR=/data02/home/philip.yang/.cbt python3 tools/temp_root.py -- python3 -m unittest bench.harness.test_run.RunTest.test_controller_failure_disposition_follows_actual_cleanup_not_failure_stage bench.harness.test_run.RunTest.test_controller_manual_pause_before_verification_has_unknown_task_truth bench.harness.test_run.RunTest.test_controller_success_requires_observed_service_exit bench.harness.test_run.RunTest.test_controller_cleanup_keeps_unwaited_service_uncertain bench.harness.test_run.RunTest.test_controller_cleanup_keeps_missing_attempt_identity_uncertain
```

Mixed controller/legacy/protected-verifier/readiness green: exit 0; 94 tests, 828.259 seconds.

```sh
CORENIUS_KATA_PINNED_BINARY=/data02/home/philip.yang/devx/tmp/controller-bench-pinned-tools/kata-pinned TMPDIR=/data02/home/philip.yang/.cbt python3 tools/temp_root.py -- python3 -m unittest bench.harness.test_run bench.harness.test_validate bench.harness.test_lead_driven
```

Default runtime green on the reviewed code tip: exit 0; 6 tests, 111.622 seconds.

```sh
env -u CORENIUS_KATA_PINNED_BINARY -u KATA_PINNED_BINARY TMPDIR=/data02/home/philip.yang/.cbt python3 tools/temp_root.py -- python3 -m unittest bench.harness.test_run.RunTest.test_controller_reference_runs_one_m01_ticket_through_controller bench.harness.test_run.RunTest.test_controller_complete_m01_records_overlap_stale_refusal_and_fresh_integration bench.harness.test_run.RunTest.test_controller_failure_disposition_follows_actual_cleanup_not_failure_stage bench.harness.test_run.RunTest.test_controller_manual_pause_before_verification_has_unknown_task_truth bench.harness.test_run.RunTest.test_controller_cleanup_keeps_unwaited_service_uncertain bench.harness.test_run.RunTest.test_controller_failed_verifier_fault_observes_real_verifier_failure
```

The default controller test gate printed PASS for normal runs and an expected FAIL for its injected verifier-failure case.
Those benchmark case outcomes are separate from the six successful unittest assertions; the FAIL was not hidden or upgraded.

Stable-path red: 1 new regression, 2 expected failures (missing native default and ignored SQLite override), 0.483 seconds.
Stable-path green: all 6 SQLite-entry tests passed, 2.133 seconds. The regression evaluates the actual controller and SQLite
modules, and the native fixture's imports, lookup and hash guard before service allocation. It checks the default and a
distinct symlink override, then rejects a corrupt native override before startup; it does not compare source strings.

```sh
TMPDIR=/data02/home/philip.yang/.cbt python3 tools/temp_root.py -- python3 -m unittest tools.agents.test_sqlite_baseline.BaselineEntry.test_pinned_kata_lookup_survives_retired_worktree_and_preserves_overrides
TMPDIR=/data02/home/philip.yang/.cbt python3 tools/temp_root.py -- python3 -m unittest tools.agents.test_sqlite_baseline
```

The affected real native fixture passed: 1 passed, 0 failed, 8 filtered out, 2.86 seconds. Runtime retained at
`/data02/home/philip.yang/.cbt/k3-_lrgoy2b`. Both overrides were unset. This deterministic fixture made no model/provider calls.

```sh
env -u CORENIUS_KATA_PINNED_BINARY -u KATA_PINNED_BINARY TMPDIR=/data02/home/philip.yang/.cbt python3 tools/temp_root.py -- cargo test --manifest-path crates/sqlite-controller/Cargo.toml --target-dir .scratch/controller03-salvage/native-target --locked --offline --features fault-injection --test kata pinned_kata_public_protocol -- --exact
```

Protected task validation passed: 41 tasks, 0 failing, 420.0 seconds. Scanner, tracker lint and whitespace gates passed.

```sh
TMPDIR=/data02/home/philip.yang/.cbt python3 tools/temp_root.py -- python3 bench/harness/validate.py
python3 tools/check_temp_paths.py
python3 tools/agents/tracker.py lint
git diff --check
```

The successful legacy-controller-v1/readiness-API smoke used
`TMPDIR=/data02/home/philip.yang/.cbt python3 tools/temp_root.py -- python3 -` with this stdin:

```python
import sys
sys.path.insert(0, 'bench/harness')
from test_run import RunTest
import results
case = RunTest()
case.setUp()
try:
    rec = case.minimal_controller_record()
    rec['controller']['schema_version'] = results.LEGACY_CONTROLLER_SCHEMA
    results.validate(rec)
    assert all(hasattr(results, name) for name in ('validate_readiness_row', 'readiness_row', 'readiness_report', 'load_readiness_rows', 'append_readiness_row'))
    print('Legacy controller v1 validates; readiness result APIs retained.')
finally:
    case.tearDown()
```

## Log identities and qualification

| Log | SHA-256 |
| --- | --- |
| `repair2-red.log` | `67aa4edef777e2cc9240355c45b244a9c0cf4196f2b594f13c4ec31a0479b149` |
| `repair2-green.log` | `79976b577adbc43c46cd0564f43ab852bff6e2c39b6957879255c248216b9408` |
| `mixed-focused-green.log` | `ccf5c1201e930932796289c600f862f49a9bdb89a0ff7022d3e58d3bea3a42be` |
| `validate-green.log` | `b360e3222a14aed6565f975f6e74da5d1966fd8438dbc9e2e028a7bc84dd6e25` |
| `scanner-green.log` | `dbb5e56649287b61dc69c5683ec286e732648207f5f2d955754a97a1f5946aa2` |
| `tracker-green.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `legacy-result-green.log` | `ceb0621ef71049e9294638e4bd2d9b368d532d087a6f1b7d0be0226100963121` |
| `pin-path-red.log` | `3b54740577ed56d744be2282bb950c7c820f00cfd67f16d9fc399082c55bd56f` |
| `pin-path-green.log` | `0f6e41d6e71464d59b9f2c3c8ce5214653e47df468a13f6591cdcdd446d3d1cf` |
| `scanner-pin-green.log` | `dbb5e56649287b61dc69c5683ec286e732648207f5f2d955754a97a1f5946aa2` |
| `default-runtime-green.log` | `1ea50bb0d62f4e44248e91f6e5c118d215ed74440d36299c51e2a721f3dbe898` |
| `native-kata-green.log` | `062baf92a6b56d973f4a60746e395095716498ebfab2a619af1648943a5ce3b3` |

The lead reports independent Standards and Spec reviews passed `ded40db69367c94f70848cae0d56b4350d9877c2`, no findings. Its detached exact-tip gate also passed:
tracker 28 tests, sync 24 tests, tracker lint, temporary-path scanner and whitespace check. Supplementary log:
`/data02/home/philip.yang/devx/tmp/corenius-salvage-a617643f/controller03-exact-gates.log`, SHA-256
`426fb2635d1a585dec5aeaa4258fb93e9f706c738841d74d8701ce69a32c562b`.
The evidence-only follow-up preserves reviewed/tested source hashes. ResourceWarnings came from unchanged capsule/tasklib
code; the suites completed successfully. The full native suite was not repeated for these Python outcome/default changes.
No live trial, broader recovery, performance superiority or product qualification is claimed. Controller-backed-bench
Ticket03 stays active, and this ticket stays claimed pending lead acceptance, landing and archive handoff. Original owner
source trees and refs were not changed. The lead owns acceptance and fast-forward landing.
