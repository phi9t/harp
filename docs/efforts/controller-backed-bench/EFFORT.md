# Controller-backed benchmark dogfooding

Status: deterministic tickets 01/02/06 completed; ticket 03 active; no live trial started.
Source read: 48d71588fff00ac70fa4a29e066a47a6b05f2ebd.

## Intent

Reuse Corenius benchmark tasks and independent grading to exercise Corenius's own controller/lead/worker collaboration path. This is a separate effort from semantic repository layout; that implementation remains paused for existing landing work.

## Accepted decisions — 2026-10-05

The user approved all four first-round recommendations:

1. Prove complete deterministic controller integration first; then require one actual lead and two actual workers before declaring live dogfooding successful. Label fixture and live results separately.
2. Bench submits the mission through the lead and owns fixtures, deadlines, declared faults and independent grading. Corenius owns admissions, execution, candidate verification, acceptance and landing. Bench does not bypass those transitions to manufacture success.
3. Controller landing is allowed only in a run-owned disposable canonical repository. Existing Corenius refs/worktrees and other user sessions remain untouched.
4. Retain the existing fleet runner as a baseline and add a separate controller mode. Use the same tasks/verifiers, reporting completion/orchestration correctness independently from time, model usage and interventions.

## Historical design-interview authority — 2026-10-05

Design interview and durable documentation only. No implementation, model-spend run, commit, push, pull request or shared-branch landing is authorized by this interview. The approved disposable landing destination is a design requirement for future authorized trials.

Later owner implementation approvals are recorded in EXECUTION.md. The current
2026-10-07 publication/landing request supersedes the interview-only Git boundary
for reviewed pending changes; it grants no new live/model run.

## Known distinction

Source inspection: the fleet runner rebases and redoes textual conflicts; the controller rejects stale-base candidates even for disjoint changes. Inference: controller-mode success expectations must distinguish fresh integration proof from fleet conflict counts. Existing retained status says complete two-worker live qualification remains unproven; no live trial has run in this effort.

## Accepted second-round decisions — 2026-10-05

Start with m01-disjoint-utils to isolate actual worker overlap and stale-base reintegration without textual conflict; add shared-interface and same-file tasks afterward. Controller mode honors declared ticket blocking edges, unlike the advisory scheduling of the retained fleet baseline; comparisons must expose that difference. First live qualification uses the existing Codex lead/workers with actual SQLite/Git/Kata integration; Trae qualification is separate. Retain successful and failed run repositories and manifests, events, candidate/proof identities, verifier output, logs and process disposition. Timeouts or uncertain effects are failed/unresolved and require survivor reconciliation, never implicit success or disposal.

## Source inspection findings

The benchmark task library already provides independent materialization and pristine-verifier support; its existing conditions are solo and fleet. The baseline launcher hardcodes a disposable task scenario and does not consume benchmark task definitions. Controller-backed task admission, results and preserved-run integration are new work. Controller proof must bind a pinned verifier and exact candidate, not a mutable worker checkout. Existing post-hoc benchmark path filtering does not prove preventive isolation. Token observations are incomplete and cannot be represented as complete spend. Current live two-worker/stale reintegration qualification remains unproven; launcher success alone is not acceptance.

## Accepted third-round decisions — 2026-10-05

First milestone is m01 end to end: deterministic proof followed by one bounded live trial. m02 dependency scheduling, m03 conflicting edits and broader recovery scenarios are follow-up milestones. Success requires final pristine task verification plus two useful overlapping worker executions, refusal of stale acceptance/landing, a fresh candidate with fresh exact verification/acceptance bindings, and recorded owned-process disposition. Manual rescue is not an autonomous pass.

Live limits: one lead, at most two concurrent workers, a 30-minute deadline, and at most one fresh integration Attempt following stale rejection. This is a trial cap, not a prediction of completion time. No automatic repeat after timeout or provider failure. Correctness gates acceptance; elapsed time, Attempts, interventions and available model usage are recorded, with incomplete usage marked unknown. No speed/cost advantage is inferred from one run.

## Final design for confirmation

- Linux-only Corenius effort, separate from the paused repository layout migration.
- Reuse benchmark task truth, protected verifiers, materialization and observation/results infrastructure. Preserve the existing fleet mode.
- Add controller-backed orchestration rather than delegating admissions, retries or landing to the fleet runner. Bench submits through the lead and independently judges the result.
- Actual controller, SQLite, Git, Kata and supervised worker execution are exercised. Deterministic scripted participants prove the integration before actual Codex lead/two-worker qualification.
- Restrict all trial changes and automatic controller landings to a run-owned disposable canonical repository with private worker repositories and a fresh dedicated session; existing sessions and Corenius refs remain untouched.
- Pin task instructions, suite and source/runtime configuration. Exact Candidate/base identities bind verification and acceptance; restored pristine tests remain independent of worker edits.
- First implementation milestone is m01 only, proving useful overlap, stale-base refusal and bounded reintegration followed by final verifier success. Broader dependencies/conflicts/recovery are later work, not implied by this result.
- Honor blocking edges as later dependent tasks are enabled; state the scheduling and stale-base policy differences in fleet comparisons.
- Retain successful/failed repositories, manifests, controller events, candidate/proof bindings, logs, final verifier results and process disposition. Deadline expiry stops further dispatch and triggers bounded reconciliation; uncertainty cannot become PASS.
- No manual rescue counted as autonomous success. One bounded actual-model trial after deterministic proof; later launches require separately authorized spend/run scope.
- Report correctness independently of time or incomplete usage. No production/permanent-baseline qualification claim follows solely from this limited task trial.

## Ambiguity review

Fleet conflict counts are not controller stale-rejection counts. A launch is not a completed trial; task PASS is not collaboration PASS. Deterministic participants do not qualify live-provider behavior. Two benchmark tickets are not necessarily the complete number of Attempts: one bounded fresh integration Attempt may follow stale rejection while concurrent capacity remains two. Retaining run state does not imply permission to leave live processes unaccounted for. The exact candidate verifier must not trust worker-modified tests. Pending existing landings require baseline revalidation before future implementation.

## Latest-state specification refresh — 2026-10-05

The user requested redoing the specification against current work. The active subject is controller-backed bench dogfooding, not the separate layout spec. Read current mainline at d7f674829e9844d011869b711d64fcf55313f7f1: tracker corrections and Sureal case study landed after the design base; no product-source changes occurred in that range. Apply current tracker lifecycle/triage conventions. Existing controller STATE/CURRENT still report qualification gaps, provider failure and consumed repair limits; these are retained snapshots, not live-process observations. SPEC.md records deterministic-before-live, exact-candidate protected grading, explicit readiness blockers, separate failure/unknown disposition and bounded run/cleanup requirements. No trial launched, no product code edited, and no source history rewritten.

## Specification published

SPEC.md is the durable local-tracker specification with lifecycle Status: open and Triage: ready-for-agent. Independent critique and adversarial review completed, with no remaining blocking specification gaps. The reviewed contract preserves both stale-acceptance and stale-landing controls; unsupported controller semantics become separately scoped prerequisites. Existing public controller and benchmark verifier seams remain the accepted test boundaries, with ticket-level test selection and final unchanged whole-task grading. Document structure/whitespace were checked; no runtime tests or launches ran. Next is blocker-linked ticket decomposition, including readiness findings before live execution.

## Owner amendment during implementation

The user explicitly selected preserving existing acceptance semantics instead of adding a stale-acceptance prerequisite. Conditional stale landing rejection and fresh integration Attempt/Candidate/proof/acceptance are required. This supersedes the earlier stronger acceptance-refusal contract. Implementation authority was granted through implement-spec with separate TraeCLI worktrees and dedicated tmux; live Codex run remains separately gated. Current canonical implementation record is on codex/controller-bench-integration, based at d7f6748.
