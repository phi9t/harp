# Acceptance cases and release evidence

Status: planned cases, none executed by this specification pass.
[plan.md](plan.md) owns delivery order; [execution-contracts.md](execution-contracts.md)
owns the contract. A case passes only with fresh behavioral evidence at its named
boundary. Use these stable IDs in qualification manifests and tracker evidence.

## Case catalog

| ID | Contract/work item | Stimulus | Required observation |
| --- | --- | --- | --- |
| A01 | C1 / H2 | Repeat admission with same key and same canonical content after reopening SQLite | Same run and one admission; changed content with same key conflicts |
| A02 | C1 / H1 | Single command, mixed agent/command plan, and a 65-node plan | First two validate under v2; oversized plan rejected before effects; v1 rules unchanged |
| A03 | C1 / H2 | Reopen schema-5 ledger through writer, then inspect; attempt read-only open before migration | Writer migrates with old data preserved; read-only never migrates |
| A04 | C2 / H1 | Bind rejected/wrong-producer output, corrupt blob or wrong schema | Consumer never starts; reason identifies binding and source |
| A05 | C2 / H1 | Fail one branch in an admitted continue-independent campaign | Strict dependents blocked; independent branches proceed; settled report retains failure |
| A06 | C3 / H3 | Kill owner between spawn and handle publication | No duplicate live job; reconcile exact identity or return unknown |
| A07 | C3 / A1,H5 | Drop SSH launch response; deliver a delayed request after client timeout | Same remote submission identity; no fresh blind launch |
| A08 | C3 / H2,H3 | Two controllers contend, then stale controller submits a decision | CAS/ownership fencing rejects stale state and effects |
| A09 | C4 / H4 | Repeated wait, invalid cursor and Ctrl-C of observer | No effects; explicit cursor error; job unaffected by observer detach |
| A10 | C4 / Q0-Q3 | Replace main agent and restart Engine with only run reference | Resume from durable evidence; zero manually redispatched healthy children |
| A11 | C4 / H4,H5 | Cancel while job is unreachable | New dispatch stops; cancellation stays pending until termination is known |
| A12 | C4 / H3,Q0 | Worker leader exits but a descendant remains | No false quiescence, workspace reuse or finalized status |
| A13 | C5 / H2 | Concurrent reservations and ambiguous job outcome | No budget overrun; ambiguous exposure remains reserved |
| A14 | C5 / H3,A1 | Supervisor disappears past job deadline | Qualified owner enforces deadline/grace; usage settled without budget reset |
| A15 | C5 / X1,V1 | Interrupt environment setup before verification marker | Partial root not reusable; resume confined staging or return incident |
| A16 | C5 / X1,V1 | Change backend/rootfs/policy after qualification | Relevant qualification invalidated; no unsandboxed fallback |
| A17 | C5 / Q1,Q2 | Execute real denied metadata write and permitted output write | Enforcement matches declared sandbox policy on actual host |
| A18 | C5 / Q3 | SSH works but allocation/device or compute probe is missing | GPU launch rejected; no inferred resource ownership |
| A19 | C6 / F2,Q0 | Interrupt after committed step 8, resume to 16 | Same deterministic CPU model/optimizer/RNG/sampler state as uninterrupted run |
| A20 | C6 / F2,Q3 | Kill during checkpoint publication | Partial checkpoint ignored; last complete compatible state restored |
| A21 | C6 / F2 | Change batch, model/code, data or optimizer while asking to resume | Incompatible resume rejected; requires a new trial |
| A22 | C6 / F2,H5 | Replay steps or reconnect log/metric collection | Physical work retained; accepted logical curve has no silent duplicates or gaps |
| A23 | C6 / H5 | Break artifact transfer; reuse output filename for changed bytes | Partial collection not accepted; digest-bound transfer/evaluation invalidated |
| A24 | C6 / H5 | Lose host with only host-local checkpoint | Report unavailable state, not a claimed recoverable local checkpoint |
| A25 | C6 / W1 | Oversized checkpoint, missing chunk or offset overflow | Bounded memory; validated chunk manifest; no global object-cap bypass |
| A26 | C7 / F1 | Seed exact duplicate leakage and label corruption | Audit finds the planted records and preserves exclusion evidence |
| A27 | C7 / F1,R1 | Invalid head mapping, future-token leak or untied weights under tied config | Numerical/semantic checks reject candidate before training admission |
| A28 | C7 / R1 | Missing seed, wrong comparison denominator or final-test tuning | No unsupported winner; report omissions and contamination |
| A29 | C7 / R1 | Hypothesis fails but all jobs and checks succeed | Execution/correctness pass; scientific conclusion negative/inconclusive |
| A30 | C7 / D1 | Crash on either side of expansion commit and replay planner output | One generation and deterministic children; no reset budgets |
| A31 | C7 / D1 | Planner requests nested expansion, new command or excess children | Rejected before effects under parent template/caps |
| A32 | C8 / H4 | Provider unavailable during failure-report generation | Engine incident report available; model-written interpretation is absent and labeled |
| A33 | C8 / C1 | Workspace lost after candidate/check publication | Read-only review reconstructs evidence; missing blobs explicit |
| A34 | C8 / W1 | Faster numerically incorrect kernel or noisy measurements | No correctness waiver; uncertainty retained; no unsupported speedup claim |
| A35 | C1-C8 / Q0-Q3 | Fresh directory with original repos/cache paths unavailable | Only declared artifact/dependency inputs used; no hidden sibling dependency |
| A36 | C3-C6 / H5 | Endpoint alias now identifies another boot/host | No observation/launch against stale job identity; explicit reconciliation incident |
| A37 | C3-C5 / Q0-Q3 | Job ends but cleanup/release acknowledgement is missing | Execution outcome retained; unresolved finalization remains visible |

C1 managed-coding also retains all 19 cases in `round-2-spec.md`; A33 is not a
substitute for its mutation, ownership, scope and candidate-verifier tests.

## Qualification sets

- Q0 local: A01-A06, A08-A14, A19-A23 using local collection where applicable,
  A26-A29 for the supplied profile, A32 and A35/A37. Later agent-authored R1 cases
  remain not_run until C1/R1 land. Numerical and real-agent cases are separate.
- Q1 Xena: repeat the relevant Q0 workload/recovery cases under Xena and add
  A15-A17. Required denials must be observed under Seatbelt, not inferred from
  a generated policy string.
- Q2 remote CPU: A07, A10-A17 as applicable to remote ownership/preparation,
  A19-A24, A35-A37, with real connection loss evidence. Local protocol cases
  remain required prerequisites even when not repeated live.
- Q3 remote GPU: Q2 prerequisites plus A18, A20-A22 on actual allocated GPU
  execution, declared numerical tolerances, and remote deadline enforcement.
- Full Ferric: relevant inherited cell prerequisites plus A25 and A34 for profiles
  that use large artifacts or optimization claims.
- Dynamic expansion: A30/A31 are required before any profile enables it.

For each bundle explicitly distinguish inherited prerequisite evidence, newly
executed cases and unavailable cases. Reuse of a prerequisite requires identical
relevant contract/backend identity; an old passing test is not a blanket waiver
for a changed adapter. Unsupported required behavior prevents cell qualification.

## Failure-injection discipline

Inject only into owned disposable jobs, directories and transport clients. Never
reboot a shared host, disable its network, revoke another user's allocation or
kill unrelated tmux sessions to simulate failure. Host-loss cases can use isolated
VM/container infrastructure when available; otherwise mark the live host-loss
case not_run and state the narrower guarantee. A modeled state-machine test and
a real host failure are separate evidence classes.

Record the injection point, confirmed precondition, fault action and observed
postcondition. An attempted kill without confirmation of the intended boundary
is not evidence that the recovery case ran. Deadline tests use short explicit
fixture limits without weakening production policy. Connection tests preserve
remote jobs and disconnect only their observer/transport.

## Completion measurements

Measure manual task dispatch, operational interventions and scientific decisions
separately. The healthy admitted workflow target is zero manual dispatch and zero
operational interventions. Deterministically recoverable injected faults need no
manual child launch; a policy-required recovery decision is allowed and counted.
Unknown ownership may correctly stop for attention. Stopping safely is conformance,
but it does not earn a claim of automatic resume for that case.

Record elapsed setup/run/recovery/collection time, actual retries, reused outputs,
physical CPU/GPU use, logical progress lost/replayed, storage retained and missing
evidence. No performance-improvement threshold is specified before measurements.
A successful research workflow can report that the proposed model change did not
help. An execution failure must never be scored as a successful negative experiment.
