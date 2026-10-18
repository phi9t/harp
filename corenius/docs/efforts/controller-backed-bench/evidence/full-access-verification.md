# Full-access verification resumed

Observed: 2026-10-05T22:12:04.573500+00:00

Full access restored. Integrated the reviewed Ticket02 product overlay and all 114 temporary-directory migration files into the owned integration worktree. Each migration file matches its reviewed after-SHA256; both independent reviewers confirmed no new integration blockers. No project commits or shared-ref changes.

The full repository gate with CHECK_BENCH=1 and CHECK_CARGO=1 is running. Its initial owned TMPDIR was too long for the native Unix transport root limit. Retain that failed evidence. The full SQLite fault-injection gate is rerunning with /data02/home/philip.yang/.cbt (owned mode 0700), and its first two native integration cases passed. The complete gate remains pending; do not resolve Ticket02 based on partial results.

Logs: .scratch/controller-backed-bench/full-access-verification/gate.log and sqlite-short-root.log. Reviewed migration manifest is retained alongside them. Gate process sessions at this checkpoint: 37491 (full gate), 93969 (short-root SQLite gate).

Dashboard moved to /data02/home/philip.yang/devx/tmp/corenius-controller-bench-dashboard and its status pane refreshed. Ticket02 shows verifying; its claim was renewed. All three existing TraeCLI processes were observed alive with completed native turns. Tickets03–05 remain dependency-blocked.

Next: inspect both exit statuses; rerun remaining failed checks under the short root, diagnose any distinct behavioral failure without bypassing isolation, and record fresh full verification before resolving Ticket02 and launching Ticket03. The long-root failure does not justify source changes or a green claim.

## Runtime repairs and focused verification

Three fixture defects were exposed after permission restoration: pagination setup raced an unrelated peer admission; Kata forced its socket beneath a checkout path exceeding SUN_LEN; imported Git fixtures referenced module-global mode instead of the existing instance field. Repaired without production behavior changes, assertion weakening, or timeout increases. Both independent review axes pass each delta.

The pagination regression now creates the peer task first but admits only the target until its actual pagination/crash assertions and restart are complete. Both topologies pass (71.05s), retaining the two normal admissions and 202 real padding threads. Kata uses checked TMPDIR; all nine tests pass (14.39s). Runtime snapshot proof passes after the instance-mode fix (34.28s). Other SQLite targets passed: Git9, process4, public20, runtime16. Prior short-root native run passed40/41; its one failure is the repaired pagination fixture.

Fresh complete deterministic m01 run passed57.578s; result retained under `.scratch/controller-backed-bench/full-access-verification/retained-complete/20261005T230736Z-controllercomplete-m01-disjoint-utils-reference-000/result.json`. Independent evidence review confirms canonical e878b9939dbe04cb356ec55a72f6b15a33435b83, tree9158b14267653941d5a87036030b8b306c41b6b1, useful overlap0.216634s, stale rejection, exactly one fresh integration proof, negative grading controls, and stopped owned processes.

Final full repository gate is running unchanged repaired source with TMPDIR=/data02/home/philip.yang/.cbt CHECK_BENCH=1 CHECK_CARGO=1. Log: `gate-final.log`; process session10809. No final green or Ticket02 resolution is claimed until this command completes. Final repair source hashes: `runtime-repair-source.json` alongside the log.

## Final adjudication

All required gate components have passing evidence. `gate-final.log` completed the full repository gate with only Team Control Plane failing on a fixture readiness race; SQLite (including all 41 native fault-injection cases), benchmark/harness/capsule, root Rust, and all other components passed. After the fixture waited for the immutable result within its existing deadline, the entire Team Control Plane gate was rerun successfully: `team-frozen-final.log`, exit 0. The assembled target passed 57 tests, with 3 existing ignored subprocess cases unchanged. This is component-wise completion; the retained original full-gate invocation remains exit 1 and is not relabeled green.

The first Team rerun passed its tests but suffered a shell EOF because root changed the wrapper while that run was using it. It is retained as failed evidence. The final rerun used frozen source and exited 0 after fixture cleanup.

Temporal now validates both TMPDIR and the effective evidence-directory override before startup. Seven temporary-policy tests pass, including actual shell rejection for both forbidden paths; positive wrapper startup and the trusted-exit test also pass. The source ban and diff checks pass. Both independent review axes accepted every final delta. No production semantics, timeout budget, authority boundary or assertions were weakened.

Verification command exits and log hashes are retained in `.scratch/controller-backed-bench/full-access-verification/verification-summary.json`. Ticket02 is qualified only for the deterministic m01 scenario; live-provider readiness remains Ticket04 and live execution still requires separate authorization.
