# Current-main health repair

Commit read: b7bee20e00c822feec48362a1dbd8d62fe9c3641.
Candidate: uncommitted codex/health-repair-20261007 in /data02/home/philip.yang/devx/tmp/corenius-health-repair-20261007.
User asked to diagnose all findings then said proceed. This is one narrow repair round for the named controller findings; previous caps are preserved. No Git publication, live trial or SQLite05/06 acceptance is claimed.

## Behavioral evidence

- Dispatcher RED: worker/effort suites and scanner twice. Corrected scheduling GREEN: five actual-shell dispatcher tests, including tracker lint RED 2 != 1 then GREEN. Default retains SQLite, parked PostgreSQL/Temporal explicitly selected by CHECK_TEAM_CONTROL_PLANE=1.
- Current projection RED: canonical worktree not registered. Historical CURRENT/STATE preserved byte for byte; new quiescent projection GREEN; eight checker tests PASS. Checker establishes projection consistency, not runtime liveness or tracker ownership.
- Peer RED: actual correction exercise with acknowledged peer timed out; presented control reached continuation. GREEN: one actual exercise test, both valid states. No native delivery frequency claim.
- Relay RED: late EOF falsely complete; stalled read 0.3003s at 0.02s deadline; trickling framing 0.3025s at 0.04s deadline. GREEN: five relay tests pass, including real HTTPResponse/socket reads and metadata privacy. Production stream sockets receive remaining timeout and absolute deadline shutdown; timer canceled/joined before socket cleanup. Upstream connection/status stage has separate existing timeout. Injected nonsocket transports are not forcibly interruptible.

## Independent review

Independent Standards reviewers covered controller, state and gate slices without reviewing their own implementation. Independent Spec review caught duplicate tracker lint; root added failing regression and fixed it; re-review PASS. Fresh-context whole-diff defect review PASS. No blocking findings.

## Verification

Focused: five dispatcher tests PASS; eight current projection tests PASS; five relay tests PASS; one peer test with two subcases PASS. Current CLI, temporary-path scanner and whitespace PASS. Full default gate is running; raw output .scratch/health-full-gate.log. Gate started before tracker lint dedup follow-up; final dispatcher suite separately revalidated. No production controller/runtime source changed afterward.

## Broader gate results

The default gate passed sync24, temp7, worktree12, worker89, effort17, landing31, acceptance18, tracker28, lead-launch48, messages7, lead-driven40, evidence15 and claims5. SQLite unit24 and client5 passed. Native Codex42 failed during process/thread creation with EAGAIN (Resource temporarily unavailable); subsequent shell dispatch could not fork. Account soft RLIMIT_NPROC is65535; observed user-slice task count63032. No assertion-level full native acceptance is claimed. Owned root Cargo parallel test run likewise exhausted resources and was gracefully interrupted by exact PID/start/group identity; serial cargo test --offline -- --test-threads=1 passed all71 root tests. Gate now uses serial root test execution, with RED/GREEN dispatcher evidence. Optional benchmark lead-driven duplicate also removed; registered default fixture remains. Monitor15, no-host-temp3 and final focused/state/scanner/whitespace gates passed separately after fork failure. Existing worker/capsule ResourceWarnings retained; suites passed.

Commands: python3 tools/temp_root.py -- sh tools/agents/check.sh (exit2); python3 tools/temp_root.py -- cargo test --offline -- --test-threads=1 (exit0). Raw full gate .scratch/health-full-gate.log. Original failed native fixture rerun was confirmed to fail thread creation with EAGAIN and then readiness timeout, not the repaired correction condition. The final isolated actual_native_control_races_and_authenticated_correction retry passed1 in98.56s (41filtered), after resource-heavy root test children exited. This corroborates the environment/concurrency failure; it does not replace a full42-native green gate. No shared processes were stopped or system limits changed.

Final independent follow-up review PASS: serial root Cargo gate and optional benchmark deduplication conform; dispatcher assertions cover both. Final dispatcher5tests PASS2.396s, current CLI PASS, tracker lint and whitespace PASS. Candidate remains uncommitted and unpublished; tracker14 remains claimed because resolve requires accepted landed Candidate. Main checkout remains clean at base.
