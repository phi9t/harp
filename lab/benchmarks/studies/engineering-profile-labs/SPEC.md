# Engineering profile learning lab specification

Source baseline: `b7bee20e00c822feec48362a1dbd8d62fe9c3641`.

## Approved learning seam

Reuse the public native fixtures in `experiments/team_control_plane/tests/kernel.rs`
and `tests/temporal.rs`. Use the current parked implementation as an experiment,
not a new controller architecture or an adoption decision. The fixture migrations
target an owned disposable database only.

## Required outcomes

| Lab | Behavior | Fresh proof |
| --- | --- | --- |
| PostgreSQL claims | Two connections issue one command concurrently | One Attempt, reservation, operation and outbox row |
| PostgreSQL receipts | Retries keep a durable disposition | Same result; changed arguments conflict; rejection remains rejected |
| PostgreSQL revision | Distinct commands race on one revision | One success and one `StaleRevision` |
| PostgreSQL atomicity | Failure at each admission write stage | No partial admission or consumed capacity |
| PostgreSQL fencing | Scope, generation, expiry and epoch restrict access | Explicit rejection after each authority change |
| Temporal wait | Canonical checkpoint precedes worker death | Native Activity history contains unsatisfied wait |
| Temporal restart | SIGKILL then replacement worker | Lost hint recovered without reapplying domain command |
| Temporal rollover | Continue-As-New changes execution Run ID | Registry incarnation advances; cursor and waits survive |
| Temporal replay | SDK replays fresh and retained histories | `WorkflowReplayer::replay_workflow` succeeds |
| Cleanup | Dispose only fixture-owned resources | Exact PostgreSQL container removed; Temporal listener closed |
| TraeCLI | Inspect CLI events and actual ACP exchange | Initialization, authentication requirement, catalog and failed live outcomes are retained |
| Codex CLI | Join the actual process outcome to one completed turn | Thread identity, ordered event interval, completed answer and zero exit agree |
| Codex app-server | Observe native RPC and retained conversation identity | Completed live turn and empty-thread resume across owned process replacement are distinct proofs |
| Native acceptance | Reject ambiguous or unexpected event streams | Model-free negative fixtures test identity, ordering, item categories and temporary-root policy |

## Runner contract

`run.py` is a learning utility, not a product daemon. It runs through
`tools/temp_root.py`, rejects unsafe temporary roots, uses a private Cargo
target, requests locked/offline resolution, and stops after a failed lab.
Each suite has an explicit 30–1800 second deadline. The default is 1200 seconds
to include a cold build. Test execution is serial between labs and libtest cases;
the concurrency tests retain their internal simultaneous clients.

For each run, retain command arrays, timestamps, exit status, test summaries,
required-test coverage, source revision/dirty state, selected input SHA-256
digests, tool versions, pinned PostgreSQL image, and cleanup assertions.
Do not capture the environment wholesale. Keep logs, database files and fresh
histories in gitignored scratch. Persist their digests and conclusions in the
tracked evidence summary.

The PostgreSQL fixture extension writes its exact resource identity and verifies
removal after its existing cleanup. It must not change database or broker behavior.
The fixture trust authentication is specific to an ephemeral loopback test service.

## Acceptance and review

A lab is integrated only after its final runner returns zero and all required
native assertions and cleanup checks pass. An earlier failed launch remains
reported. Source inspection, cached evidence and a successful build cannot
replace the fresh run. The parent context owns independent review and final
handoff; this sole-writer slice makes no independent-review claim.

## Native runner contract

`native/run_lab.py` exposes model-free inventory, schema and protocol modes,
plus explicit opt-in live modes. Protocol projections exclude credential and
configuration details before persistence. Offline sandbox probes use an ambient
read-only host closure; they do not prove hermetic or reproducible execution.
Live modes use trusted installed host runtimes and a read-only tool policy.
Retain the distinction between historical live evidence and later parser repairs;
do not repeat provider requests merely to qualify model-free acceptance changes.

## Exclusions

No production readiness, controller migration, Trondheim integration,
Insula execution, network isolation, reproducibility of the ambient host,
database HA, server restart, backup recovery, throughput or long-run reliability
is established by the database slice. That slice uses no provider calls or
real agent execution. The native slice records bounded actual provider attempts;
it does not establish active-turn recovery, lost-ack reconciliation, shared-daemon
ownership, enforced spend ceilings or successful TraeCLI inference. No GPU run
or production adoption is part of either slice.

## Native runtime companion

The integrated native walkthroughs and evidence are at
[native/README.md](native/README.md). Their qualification is independent of the
PostgreSQL and Temporal results.
