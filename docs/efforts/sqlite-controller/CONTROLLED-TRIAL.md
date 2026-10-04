# Controller-first controlled trial

Status: owner-requested goal and qualification contract; not implemented or executed.
Priority: highest, within the retained local baseline. Source read: a21316f; baseline spec commits9d90222/d18e413. This refines SPEC.md and ticket06; it does not authorize wider production claims.

## Goal

Implement the Corenius controller first, prove its state machine and SQLite/Git/Kata/runtime integrations in a bounded local setting, then demonstrate the entire team loop in a new dedicated tmux session containing one Codex lead, two Codex workers, the controller and the dashboard. An outer harness submits missions to the lead through tmux and judges outcomes using controller events, immutable artifacts, actual processes and Git state. Retain this harness and supported baseline to build richer versions afterward.

## Build order

1. Controller command/state-machine core and real SQLite persistence: authenticated/idempotent commands, revision guards, two-slot admission, messages, operations and event projections. Test without models or tmux.
2. Actual Git private repositories and conditional landing; isolated Kata deployment/project via supported API; runtime adapter and independent supervisor. Exercise these against real local services/processes with deterministic workers before paid/live model runs. Fixture proof is explicitly labeled.
3. Dedicated bounded live Codex trial, lead/control-center interaction, concurrency, failure injection and independent evidence adjudication. No unrelated orchestration feature may displace this sequence.

## Dedicated topology

New session name: corenius-baseline-<run-id>. Never reuse hack, vision or an existing user session. Capture exact tmux session/pane IDs and process identities in a run manifest.

Windows: 0 harness/status; 1 controller; 2 dashboard; 3 lead; 4 worker-a; 5 worker-b. The outer harness is independently supervised outside this session; window0 observes it. Controller window observes/attaches a supervised controller service. Worker windows attach controller-owned executions; they are not alternate schedulers. The dashboard may be terminal-based initially, but must support the SPEC's required views and actions. UI attachment loss must not imply execution loss.

A unique run root contains the SQLite database, controller artifacts, canonical Git repository and two private task repositories outside the human source checkout. Kata uses a unique sandbox project/work item namespace with no production task mutations. Pin source/build, schema, Kata API/deployment, Codex runtime, toolchain and test suite versions in the run manifest. No host credentials, database or canonical repo exposure inside worker environments.

## Outer harness authority and protocol

Harness may create its own session/project fixtures, submit a mission to the lead through its identified tmux pane, inject declared faults into its own processes, and collect evidence. It cannot assign tasks, fabricate worker results, accept candidates, update protected refs or patch controller storage to force success. Those actions must occur through normal lead/operator controller commands. Explicit scripted operator acceptance is permitted only through the authenticated operator command path and is recorded separately from the lead.

Submission uses a saved UTF-8 prompt and literal bracketed paste followed by an explicit submit; shell evaluation/interpolated commands are forbidden. Correlate prompt, mission, tasks, attempts and events by a run ID. Capture delivery and controller admission separately: visible terminal input is not acknowledgement. Stop or report a transport fault if delivery cannot be established; never repeatedly submit an uncorrelated mission. Agent-facing controller tools carry structured requests; terminal text is a diagnostic trace, not the result oracle.

Harness waits on controller event sequence/queries and independently observes processes/ref identity. Use deadlines, allow-listed owned-process fault injection, no blind sleeps or terminal-text success matching. Persist a machine-readable result, event range, candidate/base hashes, verification/acceptance bindings, process observations and log digests. Live trial has explicit wall-time and repair limits, without implementing a general budget system. A timeout is failed or unresolved, never implicit success; retain the run and reconcile survivors.

## Scenarios and acceptance

Happy path: harness asks lead to complete a small change with two independently testable tasks. Lead records pinned Kata intent and assigns both through controller. Observe both worker executions overlapping; each uses a private repository. Workers ask/report/submit through Corenius. Exact candidates get isolated verification and explicit acceptance. Landing proceeds through the protected serialized gate. Final combined checks pass, Kata receives the recorded dispositions, dashboard reflects the same mission and artifacts, and cleanup disposition is visible.

Concurrency: both candidates originate at the same base. First lands; second is rejected as stale and reworked on the new base into a new candidate. Old evidence cannot accept the new candidate. Prove duplicate admission/submission/land commands cannot duplicate effects and no third worker is admitted while both slots are occupied, including an unknown execution.

Recovery: disconnect views; restart controller with workers alive; replace lead from durable context; lose a start acknowledgement; crash after a ref update before confirmation; request cancel near submission; disconnect Kata and replay synchronization. Each must recover safely or expose a precise unknown state that blocks conflicting actions. Record separate backup/restore refusal or successful reconciliation proof under SPEC. Deterministic faults must be proven first; live trial need not spend model calls reproducing every storage fault.

Boundary: attempt writes canonical refs, accesses another task repo/host secrets, signals controller and submits with revoked identity. Each prohibited action fails without damaging existing work.

PASS requires actual SQLite/Git/Kata integrations and at least one actual Codex lead + two Codex workers run, not three transcripts or runtime mocks. Independent review checks evidence against scenario requirements. Unsupported guarantees are explicit and prevent corresponding readiness claims. Retain the one-command launch/run procedure, fault suite, operating guide and baseline configuration in the repository.

## Exclusions

Trae/Claude qualification, PostgreSQL/Temporal, multiple hosts/controllers, general Plan IR, financial budgets and jj support are not prerequisites for this Git/Codex controlled trial. jj remains qualification ticket07 behind the common workspace interface. No production deployment, shared branch rewriting or broad user-session cleanup.
