# Supervised workflow workstream

Status: implementation in progress after the owner's “proceed to impl” instruction.
The unmerged inspection baseline is `f6feae04`. V2 contracts, verified bindings,
SQLite job state and native backend protocol work are under focused verification;
these do not yet provide an executable v2 CLI. Kata `dfv4` tracks contracts,
`sx7w` tracks state, and `zj1e` remains the workstream record.

The standalone research fixture now exercises actual CPU training and checkpoint
recovery. It is separately versioned and is not a Harp build dependency. Q0 has
not passed: the existing macOS lifecycle mechanisms cannot establish C3's required
full-tree quiescence and independent deadline guarantees after owner death.

## Read in this order

1. [Implementation plan](plan.md): scope, dependencies, owned files, work items and
   verification commands. This is the only delivery order.
2. [Execution contracts](execution-contracts.md): admission, dataflow, job ownership,
   environment/resource accounting, checkpoints, cancellation and compatibility.
3. [Workload/environment matrix](workload-environment-matrix.md): separately
   qualified workload and environment profiles, including unsupported/future cells.
4. [Research scenarios](frontier-research-scenarios.md): precise tiny Ferric fixture,
   scientific controls and data/architecture/training/evaluation scenarios.
5. [Acceptance matrix](acceptance-matrix.md): fault cases and qualification evidence.

Supporting module specifications:

- [Coding/review scenarios](supervised-workflow-scenarios.md) retain patch review,
  incident diagnosis, bounded repair and evidence-claim auditing.
- [Managed coding candidates](round-2-spec.md) defines workspace mutation,
  candidate publication and verification. It applies to repository-writing agents,
  not to restoring training weights.
- [Implemented inspection design](design.md) records the existing read-only
  inspect/explain increment and must not be read as claiming the later work exists.

## Decisions this proposal makes

| Concern | Decision |
| --- | --- |
| Initial useful workload | A real tiny CPU data/training/evaluation workflow using supplied models, followed by independent agent analysis/review |
| Complexity growth | Hold work fixed while qualifying local, Xena and remote Vaso; then expand workload profiles |
| Architecture-editing dependency | Qualify managed coding before agent-authored edits, but do not block fixed-model training on it |
| Orchestration owner | Harp Engine; workload frameworks retain internal execution; Armada retains host/process operations |
| Remote ambiguity | Reconcile durable submission identity; unknown is not permission to relaunch |
| Recovery promises | Qualify observer loss, worker death and host loss separately |
| Experimental changes | New trial for changed code/data/model/optimizer/batch; retry preserves scientific identity |
| Final test | All three final seed checkpoints of each validation-selected/tied configuration; no test-based seed or tie selection |
| Calibration | Fixed bounded bootstrap after pins, then final profile after the trainer exists |
| Larger checkpoints | Chunked artifacts before large-model profiles, preserving the current object cap |
| Runtime graph growth | Fixed v2 graphs first; one bounded persisted expansion generation later |
| Deferred work | Real distributed execution and implementation details for uninspected framework profiles |

## Next executable slice after adoption

P0 freezes imported inputs, dependency artifacts and bounded calibration authority.
F1/F2 build the standalone workload. H1-H4 build Harp's v2 command/dataflow/lifecycle
path. F3 freezes the calibrated executable profile. Q0 proves the combined local
workflow with real training, a real agent review, process interruption and a
replacement supervisor. This slice must leave a complete evidence bundle, not
only a successful command transcript.

## Current implementation boundary

| Area | Implemented increment | Remaining qualification |
| --- | --- | --- |
| H1 | Strict v2 parsing, canonical identity, DAG/resource validation, schema-checked artifact and terminal-outcome bindings | Lower into Engine scheduling; immutable environment/executable and materialization authority |
| H2 | Versioned admission and guarded SQLite job lifecycle under review | Integrate accepted outputs, receipts and runtime effects through Engine |
| H3 | Typed request-bound native job protocol, separate collection, operator-approved qualification preflight | Actual durable owner and backend conformance; no native backend enabled |
| F1/F2 | Standalone data/model/AdamW/checkpoint/evaluation fixture with real process-kill tests | Combined Harp campaign, real provider interpretation and supervisor replacement |
| F3 | Bounded CPU calibration and proposed profile | Caller admission and qualified execution backend |
| H4/Q0 | Existing v1 inspect/explain remains available | V2 submit/drive/wait/cancel and end-to-end qualification |

Source review found that ordinary process-group cleanup is insufficient: owner
loss can leave a guest alive, and a descendant can leave the original process
group. The existing Xena hardening tests explicitly document both cases. The new
[real process counterexample](../../../crates/harp-cli-process/tests/jobs.rs) also
passed locally: a bounded detached child survives cleanup of its original
process group. A
leader PID disappearing therefore cannot authorize terminal acceptance, refund
or relaunch. Native workflow preflight requires containment, owner fencing,
idempotent submission and independent resource enforcement; it must reject an
unqualified backend. Resolving this requires a separately qualified lifecycle
boundary, followed by the original H3 fault tests. The scientific fixture's
checkpoint recovery tests do not establish that process-lifecycle guarantee.
