# 01 single-ticket-controller-path: Run one benchmark ticket through the controller

Type: task
Status: resolved
Owner: trae-controller-bench-01
Blocked by: 06
Claimed:

Triage: ready-for-agent

## Acceptance

- [x] Preserve existing solo/fleet modes; controller mode rejects unsupported tasks and dependency graphs before execution.
- [x] Pin source/build, task instructions, role identities, verifier, toolchain and isolated Kata configuration; correlate mission delivery acknowledgement with durable admitted intent.
- [x] Use actual SQLite, Git, Kata, supervisor and Insula integration; the outer harness does not assign workers, forge outputs, accept candidates or land refs.
- [x] Create fresh run-owned repositories and session; existing user sessions, Kata projects, Corenius mainline and other worktrees remain untouched.
- [x] Initial string-ticket proof uses the existing complete title-case test unchanged; numeric-ticket proof uses the existing complete clamp test unchanged. Pin test selection, suite digest and exact Candidate/base.
- [x] Export exact committed content into the isolated verifier view with pinned protected test inputs and adapter/toolchain identities; mutable ambient harness imports and worker-modified tests cannot establish proof.
- [x] Prove each partial contract fails on the corresponding unfixed/noop content and succeeds on its independently valid fix; do not require unfinished whole-task checks to pass at first landing.
- [x] Capture process identities, actual proof/acceptance bindings and disposable canonical commit identity; basic retained outcomes are available without any model calls.
- [x] Preserve relevant benchmark/controller tests and document the deterministic run procedure and qualification limits.

## Parent

Controller-backed benchmark dogfooding: first milestone specification, same durable effort. Source inspected at d7f674829e9844d011869b711d64fcf55313f7f1; implementation uses the reviewed integration revision containing completed blockers.

## What to build

A deterministic lead submits one m01 ticket through normal role interfaces and the actual controller admits, supervises, verifies, accepts and lands its exact candidate in a retained disposable repository.

## Exclusions

No two-worker completion claim, live model trial, stale-control redesign, permanent readiness or repository layout changes.

## Verification

Behavioral red/green at benchmark CLI and authenticated controller seams; real deterministic service/worker integration; partial verifier controls, pinned artifact records and exact disposable landing. Verify clean or explicitly retained owned-process disposition.

2026-10-05T08:38:15Z implementation evidence, without resolving the ticket:

- RED: `python3 -m unittest bench.harness.test_run.RunTest.test_controller_reference_runs_one_m01_ticket_through_controller bench.harness.test_run.RunTest.test_controller_mode_rejects_unsupported_tasks_before_execution` exited 1 as expected before implementation; `controller:t1` was rejected by the legacy `solo|fleet:N` condition parser. Log: `/tmp/corenius-controller-bench-implementation-20261005T075523Z/01/red-controller-tests.log`.
- GREEN: `python3 -m unittest bench.harness.test_run.RunTest.test_controller_reference_runs_one_m01_ticket_through_controller bench.harness.test_run.RunTest.test_controller_reference_runs_numeric_m01_ticket_through_controller bench.harness.test_run.RunTest.test_controller_mode_rejects_unsupported_tasks_before_execution` exited 0. Log: `/tmp/corenius-controller-bench-implementation-20261005T075523Z/01/focused-controller-tests-06.log`.
- GREEN: `python3 -m unittest bench.harness.test_run` exited 0, 20 tests in 31.199s. Log: `/tmp/corenius-controller-bench-implementation-20261005T075523Z/01/full-bench-test-run-03.log`.
- GREEN: retained deterministic controller CLI runs for `controller:t1` and `controller:t2` exited 0. Results are under `/tmp/corenius-controller-bench-implementation-20261005T075523Z/01/green-controller-out-03/`; summary is `/tmp/corenius-controller-bench-implementation-20261005T075523Z/01/green-controller-results-03-summary.json`.
- GREEN: `timeout 180s python3 bench/harness/validate.py --task m01-disjoint-utils --json /tmp/corenius-controller-bench-implementation-20261005T075523Z/01/bench-validate-m01-01.json` exited 0 with `ok m01-disjoint-utils 9 checks`. Log: `/tmp/corenius-controller-bench-implementation-20261005T075523Z/01/bench-validate-m01-01.log`.
- GREEN: `cargo test --manifest-path crates/sqlite-controller/Cargo.toml --target-dir .scratch/controller-backed-bench/target --locked --offline --features fault-injection --test git actual_private_git_candidates_verification_landing_recovery_and_hygiene -- --nocapture` and `cargo test --manifest-path crates/sqlite-controller/Cargo.toml --target-dir .scratch/controller-backed-bench/target --locked --offline --features fault-injection --test kata pinned_kata_public_protocol -- --nocapture` exited 0. Logs: `/tmp/corenius-controller-bench-implementation-20261005T075523Z/01/sqlite-controller-git-private-fixture-01.log` and `/tmp/corenius-controller-bench-implementation-20261005T075523Z/01/sqlite-controller-kata-pinned-fixture-01.log`.
- BOUNDED STOP: full `python3 bench/harness/validate.py` was interrupted after a long no-output wait, exit 130. Log: `/tmp/corenius-controller-bench-implementation-20261005T075523Z/01/bench-validate-01.log`.
- OPEN: runtime-closure acceptance remains unchecked. Current Git backend verifier isolation pins the verifier adapter, protected test bytes and configured executables, but ro-binds ambient `/usr`. Full closure qualification requires a pinned independently owned runtime snapshot actually mounted into the verifier, with its manifest bound to the proof; a manifest alone is insufficient. This prerequisite awaits owner scope approval.

## Authority and execution

Linux only. Use one writer per isolated worktree and normal controller role interfaces. Preserve existing work, consumed repair authority and historical evidence. Ticket publication authorizes no product execution, model spending, commits, pushes, pull requests or shared-ref landing; advance through the subsequent explicitly requested execution phase. Review repository Standards and parent Spec; a ticket result cannot broaden qualification beyond its stated scenario.

## Comments
- 2026-10-05T07:56:21Z trae-controller-bench-01: claimed until 2026-10-05T11:56:21Z
- 2026-10-05T08:38:15Z trae-controller-bench-01: implemented deterministic `controller:t1`/`controller:t2` benchmark path and retained evidence; closure item remains unchecked pending owner-scoped runtime snapshot work.

## Runtime prerequisite completion

Ticket06 supplies the pinned independently owned verifier runtime mounted read-only into the actual verifier; its manifest binds to proof. Fresh retained t1/t2 controls and exact-candidate landings pass; independent Standards/Spec re-review has no blockers. See evidence/runtime-review.md and implementation-06/report.md. Earlier ambient-runtime limitation is superseded by this scoped fix.
- 2026-10-05T18:01:33Z trae-controller-bench-01: resolved
