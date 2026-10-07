# Controller-backed benchmark dogfooding: first milestone

Status: open
Triage: ready-for-agent
Type: specification
Source revision: d7f674829e9844d011869b711d64fcf55313f7f1
Design-worktree base: 48d71588fff00ac70fa4a29e066a47a6b05f2ebd

## Problem Statement

Corenius has validated multi-ticket benchmark tasks and a retained controller prototype, but benchmark fleet runs orchestrate work themselves. They do not establish that the Corenius lead/controller path admits, supervises, verifies and lands useful concurrent work. Existing live evidence includes a solo success and a later provider-overload/timeout failure; useful two-worker overlap and stale-base reintegration remain unproven. A successful launcher, visible panes or final passing files cannot substitute for that proof.

Developers need a repeatable, independently graded task that demonstrates the actual collaboration system while retaining honest failure and qualification evidence.

## Solution

Add a distinct controller-backed benchmark mode, preserving current solo/fleet modes and task/verifier definitions. First support m01-disjoint-utils on Linux: real SQLite/Git/Kata and supervised execution with deterministic scripted participants, followed by one separately authorized live Codex lead/two-worker trial.

Bench supplies fixtures and submits the mission through the lead. The product path owns admissions, execution, candidate verification, acceptance and conditional landing. Bench observes and independently grades a retained run-owned repository, never landing into Corenius mainline. Task correctness and collaboration correctness are separate verdicts.

## User Stories

1. As a developer, I want to run an existing validated task through Corenius, so that I test the product rather than another scheduler.
2. As a benchmark maintainer, I want existing solo/fleet behavior preserved, so that prior results remain interpretable.
3. As a reviewer, I want deterministic participants first, so that integration defects are reproducible before model calls.
4. As an owner, I want one real lead and two real workers in the live trial, so that fixture results cannot masquerade as live collaboration.
5. As a lead, I want benchmark instructions admitted through normal tools, so that task intent and authority remain durable.
6. As a worker, I want a private repository and scoped tools, so that work stays isolated from the canonical repository and other tasks.
7. As an owner, I want proof of useful execution overlap, so that two occupied slots alone do not imply collaboration.
8. As a verifier, I want protected test inputs bound to the exact candidate, so that worker edits cannot manufacture acceptance.
9. As a reviewer, I want stale-base transitions rejected and retained, so that disjoint edits do not hide stale acceptance.
10. As a lead, I want a bounded fresh integration Attempt, so that rework receives fresh proof rather than recycled acceptance.
11. As an owner, I want disposable-only automatic landing, so that qualification cannot mutate my working branches.
12. As an observer, I want correlated durable events and process identities, so that panes and transcripts are not the success oracle.
13. As an operator, I want bounded runs with explicit failure and unknown states, so that a timeout does not lose execution accountability.
14. As a reviewer, I want successful and failed runs retained, so that results can be independently adjudicated.
15. As a maintainer, I want existing open hardening and qualification limits preserved, so that this benchmark does not silently reset repair authority.
16. As an evaluator, I want task and orchestration verdicts separate, so that a correct tree cannot conceal bypassed control transitions.
17. As an evaluator, I want recorded interventions and available usage, so that incomplete metrics cannot support misleading cost claims.
18. As a developer, I want the same task truth across controller and fleet modes, so that measured differences have identifiable causes.
19. As a maintainer, I want unsupported dependency tasks refused, so that advisory fleet semantics are not silently copied into controller mode.
20. As an owner, I want precise readiness prerequisites for live execution, so that unaccepted controller work is not presented as qualified infrastructure.

## Implementation Decisions

- Scope is current Corenius, Linux only, independent of the semantic-layout migration. Adapt existing benchmark materialization, task metadata, protected verifier, observer/result and tmux infrastructure; do not replace the fleet implementation or reuse its scheduler/landing functions as controller orchestration.
- Pin actual source/build, task/ticket instructions, test suite, runtime/provider/model, toolchain and Kata deployment/API identities in a run manifest. Model selection must propagate consistently to actual lead/worker dispatch, not merely appear in the manifest. Do not hardcode an obsolete model from historical evidence.
- First support only the independent two-ticket task. Preserve allowed-path restrictions, verifier truth and task inputs. Shared-interface, same-file and dependent-ticket execution are later milestones. Refuse unsupported tasks/dependency graphs explicitly; future dependent-task eligibility must wait for declared prerequisite acceptance.
- Create one new dedicated session and unique run root, isolated Kata namespace, disposable canonical repository and private complete worker repositories. No reuse of existing user sessions, production Kata work items or shared refs. Use the controller's existing supervisor and Insula boundary; do not replace enforcement with benchmark post-hoc patch filtering.
- Submit saved literal mission input through the identified lead entry point. Correlate delivery acknowledgment, admitted intent, tasks, Attempts, Executions, events and results. The outer harness may prepare fixtures, query observations and run independent grading; it may not assign workers, forge results, patch state, accept candidates or move protected refs to force success.
- Deterministic participants act through the same normal authenticated role interfaces as live participants. Real storage, Kata/Git integration, supervised execution, candidate capture, verification, acceptance and landing remain in the path; scripted transcripts or direct database manipulation are insufficient.
- Define separate pinned verifier contracts for partial and final work. For m01, initial string-ticket verification runs the existing complete title-case test, and initial numeric-ticket verification runs the existing complete clamp test; keep their assertions unchanged. Each partial proof binds its ticket, Candidate/base, selected test identity and suite digest. Applying neither fix must fail each respective contract. The integration Candidate and final canonical snapshot must pass both unchanged tests through the whole-task verifier. The first landing is not expected to pass the incomplete whole task.
- Bind controller verification to an immutable exact Candidate/base and pinned independently owned pristine suite. Protected verifier inputs are unavailable for worker modification. Do not mutate the canonical tree after landing to repair tests before grading; export the exact committed Candidate or canonical commit/tree into an isolated verifier view, restore only the pinned independently owned protected test inputs there, and retain both source and verifier-view identities. Pin the adapter/script and toolchain closure used inside that view; imports from mutable ambient harness source cannot establish exact proof. Empty patches, omitted tickets and modified tests must not create success.
- Admit two useful initial worker Attempts from the same canonical base and demonstrate actual overlapping work, not merely overlapping pane or slot lifetimes. Observe original candidate/base identities. Land the first valid candidate through normal authenticated authority, exercise stale rejection on the second, and record the rejecting transition. Require a deterministic control that attempts new acceptance after the base becomes stale and observes rejection, plus a stale landing attempt that observes rejection. Acceptance issued earlier while the base was current remains valid historical evidence but does not satisfy the stale-acceptance control. Prove that old proof cannot authorize changed candidate/base content. If current controller semantics cannot enforce a required refusal, record a prerequisite gap for separate scoped product work rather than relaxing this contract or hiding a controller redesign inside the bench adapter.
- Permit at most one fresh integration Attempt after stale rejection; it starts from current canonical state and retains needed task changes. Fresh candidate verification and acceptance bind its new identity. At most two worker executions may occupy concurrent capacity, including unresolved executions. Do not bypass release/cancellation to make room.
- Keep current protected-ref conditional landing and role boundaries. Automatic landings are authorized only in the disposable benchmark canonical repository during separately authorized execution. The deterministic and live outcomes are separately labeled, with no general permanent-baseline qualification claim.
- Readiness is not inferred from source availability. Before live launch, require deterministic integration success, exact-source relevant tests/reviews and explicit disposition of current controller hardening/qualification blockers. Existing consumed repair rounds and unreconciled effects remain in force; this effort does not grant repair resets, authorize product release, or reuse unfinished live deployments. Integration-only work may proceed in its own bounded ticket without claiming underlying controller acceptance.
- Live configuration is one Codex lead, at most two concurrent workers, one trial, and a 30-minute execution deadline measured from durable run admission immediately before first mission submission; it includes lead/model startup after admission, worker activity and candidate/final verification. Record fixture/build preflight separately with bounded stage timeouts; a failed preflight must not consume model calls or enter execution. Record the dispatch cutoff and cleanup separately, so setup or grading cannot silently reset the clock. No automatic repeat after timeout or provider failure. A retry within the approved run must respect the single integration Attempt limit and existing admitted operation semantics; broader provider retries are not added.
- At expiry or failure stop further dispatch through normal authority, perform bounded owned-process reconciliation and preserve uncertain effects. A terminal execution deadline is not a promise of immediate physical cleanup. Record cleanup outcome and remaining unresolved identities; never kill unrelated processes or delete evidence. Exact stage and cleanup timeout values belong in execution tickets and must be fixed before a run.
- Retain manifests, canonical/private repositories, event ranges, Candidate/base/proof/acceptance bindings, verifier input/output identities, logs/digests, process observations and dispositions for success and failure. Exclude credentials and secret payloads from public records. Raw artifacts may remain private with redacted durable summaries and content identities; source-relative scratch directories are not the only authoritative location for results others rely on.
- Introduce a versioned controller result contract integrated with result validation, loading and reporting, preserving readability and interpretation of existing solo/fleet records. The legacy condition validator and boolean passed field alone cannot encode the new mode or unknown lifecycle state. Keep task verdict, orchestration verdict and effect disposition independent; known FAIL remains recorded even when effect disposition is UNRESOLVED.
- Results include deterministic/live mode, task outcome, orchestration outcome, failure stage, resolved/unresolved disposition, interventions, Attempt counts, elapsed setup/execution/cleanup times and available model usage. Unknown or partial usage stays explicitly unknown/partial; do not synthesize totals from output tails. Manual rescue disqualifies autonomous PASS even if the final task passes.
- Preserve fleet comparison results under their actual configuration. Fleet textual-conflict counts are not controller stale-rejection counts. Correctness gates initial acceptance; no speed, cost or recovery superiority is required or inferred from one run.
- Use current durable local-tracker conventions: lifecycle status and triage are separate. Specification readiness is not live-run, model-spend, Git publication or merge authorization. Revalidate against the integration source revision before implementation because the design worktree predates recent documentation landings.

## Testing Decisions

- Reuse the existing benchmark task-library/materialization/verifier seam and controller public authenticated command/query seam. These are the accepted design's existing seams; no new test-only product API is required.
- First demonstrate a deterministic end-to-end m01 run through actual controller, supervisor, SQLite, Git and isolated Kata. Assert two useful executions overlap using correlated process/controller observations and actual task work. Record the first landing, both stale-acceptance and stale-landing rejection (the acceptance negative control may be a separate deterministic fixture), fresh integration proof, final pristine verification and identity-bound process disposition.
- Verify adversarial grading controls through existing benchmark validation prior art: initial/noop state fails, omission of either ticket fails, and worker test tampering does not affect independent grading. Use fixed suite/candidate identities rather than source-string assertions or mocks of the product logic.
- Exercise task-mode dispatch, invalid/unsupported tasks, missing prerequisites, delivery failure, provider failure, timeout, manual intervention and missing evidence classifications. Failure must never yield autonomous PASS. Failure propagation and cleanup can use controlled deterministic fault fixtures without claiming broad recovery qualification.
- Preserve existing benchmark harness and tmux tests and controller integration checks affected by the adapter. Use the exact runtime/source candidate for verification. Existing open defects must be reproduced/dispositioned explicitly, not hidden by weaker tests or longer waits.
- After readiness is proven and actual spend/run scope authorized, run one bounded live trial with actual Codex lead and two workers. Require the same independent task and orchestration evidence; record any limitation. No live run has been executed for this effort.
- PASS requires both task and orchestration checks, no manual rescue, and verified lifecycle closure or explicitly intended retained service disposition with no uncertain worker effects. Missing evidence or prerequisites blocks qualification; failures with uncertain effects remain unresolved until reconciled. A failed task with cleanly stopped processes remains failed, not unresolved solely because it did not pass.
- Independent review adjudicates retained evidence against this contract. Unit tests, reference patches, model probes, launcher exit zero, occupied slots and terminal text are individually insufficient.

## Out of Scope

Repository layout changes; sibling repositories; fleet replacement; Trae/Claude controller qualification; all benchmark tasks; shared-interface/conflicting/dependent tasks in the first release; broad crash/restore/lead-replacement qualification; production deployment; multi-host execution; PostgreSQL/Temporal roadmap revival; general budgets; automatic trial repeats; repair-cap resets; model spend without authorization; automatic Corenius mainline landing; performance superiority claims.

## Further Notes

Current mainline has landed tracker-guidance corrections and an observed Sureal collaboration case study since the original design baseline. Those commits change documentation, not product source. The case study is process evidence, not controller qualification: panes, worker completion text and ad hoc merges cannot replace supervised exact-candidate evidence.

Current retained controller status is a snapshot: accepted tickets 01–04, ticket 05 unaccepted with open peer-wait/relay hardening and consumed repair authority, and ticket 06 live qualification pending after provider overload and outer timeout. Historical solo success does not establish two-worker overlap or reintegration. Recheck exact-source status before execution; do not treat these documents as live process inventory.

The requested refresh synthesizes the accepted design and latest inspected state. Runtime tests were not run for this document. Subsequent work is independent critique reconciliation, blocker-linked tickets and bounded isolated implementation, with separate live-run authorization.

## Independent critique reconciliation

Fresh-context inspection at the stated current mainline required scoped controller-readiness disposition, exact committed-content grading, partial/final verifier distinction, explicit versioned result integration, enforceable deadline semantics and retained FAIL/UNRESOLVED evidence. Those contracts are incorporated above. The Sureal case study supplies negative examples: mutable completion text, pane liveness and uncommitted-tree checks cannot prove exact controller collaboration. None of this inspection constitutes runtime verification.

Independent adversarial specification review found no remaining blocking gaps after the stale-acceptance contract was restored. This is specification review only; all implementation/runtime qualification remains unverified.
