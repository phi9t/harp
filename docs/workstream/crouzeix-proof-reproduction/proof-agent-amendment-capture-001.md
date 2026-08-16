# Proof Agent Amendment Capture 001

## Claim Ceiling

This capture records proof-agent control-plane guidance from amendments `001`
through `012`. Amendments `005` through `012` remain the active normalized
handoff for CPFR-R014; amendments `001` through `004` contribute baseline
facts and superseded early instructions. This is not a proof of Crouzeix's
conjecture, not a provider result, and not a substitute for runtime tickets,
run artifacts, tracker state, or formal receipts.

## Active Handoff Rule

The next proof-reproduction implementation agent must claim `CPFR-R014` and
implement the provider-free runtime repair before any CPFR-032 continuation.

No live proof-agent execution is allowed from `expert-frontier-001` until
`CPFR-R014` reaches terminal tracker state with verification evidence.

Forbidden until `CPFR-R014` is terminal:

- root continuation;
- mutating `recover-roots`;
- `evaluate-roots`;
- generation execution;
- finalization;
- CPFR-033 and downstream phases.

## Unchanged Proof State

The amendment chain consistently records the same live frontier state:

- `expert-g0-function-theory` is terminal `failed` through recovery evidence.
- `expert-g0-operator-dilation` is nonterminal `running`.
- There is no `operator-dilation` provider call, attempt, admission,
  mathematical node, frontier event, or attempt-ledger row.
- `expert-g0-matrix-extremal`,
  `expert-g0-completion-positivity`, and
  `expert-g0-approximation-audit` are still pristine.
- Context leakage scans report zero forbidden findings.
- Runtime validation fails because `expert-g0-operator-dilation` is not
  terminal.

These facts are control-plane facts. They do not decide mathematical truth.

## Baseline Facts From Amendments 001 Through 004

The early amendments add still-active context that is narrower than the
CPFR-R014 handoff:

| Amendment | Still-active fact | Evidence boundary |
|-|-|-|
| 001 | Ignored `.runs/` artifacts are primary evidence; do not rewrite run evidence to satisfy validators; provider transport success is not proof success; control/resource failure is not proof failure. | Run artifacts, runtime tickets, and tracker evidence outrank agent closeout prose. |
| 001 | The flat baseline produced a historical negative signal: three flat routes converged on the positive-boundary-measure / conjugate-Cauchy family, indicating route-family collapse pressure. | Historical baseline context only; it is not proof progress or disproof. |
| 002 | `expert-g0-function-theory` was recovered as terminal `failed` with `provider_invoked=false`, one failed attempt, no admissions, and no mathematical nodes. | Existing `expert-frontier-001` control-plane evidence. |
| 003 | `expert-g0-operator-dilation` was a second nonterminal root with no per-ticket call, attempt, admission, mathematical node, frontier event, or attempt-ledger row. | Per-ticket evidence, not aggregate `run_receipt.json`, decides orphan classification. |
| 003 | The remaining `matrix-extremal`, `completion-positivity`, and `approximation-audit` roots were pristine in the recorded state. | Existing `expert-frontier-001` root-ticket state. |
| 004 | The exact blocker was local pre-provider ticket-scope validation, not a provider invocation. | `_run_expert_attempt()` recorded `admitted -> running` before provider-boundary validation. |

## Superseded Early Instructions

These early instructions are no longer active guidance:

- Amendment 001's claim that `expert-g0-function-theory` was the current
  orphaned `running` ticket is superseded by later recovery evidence.
- Amendment 001's and Amendment 002's resume/recovery instructions are
  superseded by the CPFR-R014 provider-free repair gate.
- Amendment 002's guidance to resume the four pristine roots is superseded by
  the stricter rule that CPFR-032 continuation is forbidden until CPFR-R014 is
  terminal with evidence.
- Amendment 003's diagnosis is superseded where Amendment 004 identifies the
  concrete local `forbidden_sources` provider-boundary validation failure.
- Any early wording that frames the next action as operational root recovery or
  root continuation is superseded by the provider-free CPFR-R014 repair
  contract below.

## Runtime Root Cause

The blocked state is caused by a local provider-boundary failure being recorded
as if a provider-scoped root ticket had started. The repair must move
provider-scope validation before any transition from `admitted` to `running`.

The repair must also classify the existing nonterminal `operator-dilation`
ticket without inventing provider evidence, mathematical output, or admission
events.

## CPFR-R014 Repair Contract

### Scope

`CPFR-R014` is provider-free. It must repair runtime state handling and
readiness validation. It must not execute live root experts.

### Required Test Surface

The repair must include tests for at least these behaviors:

1. A provider-boundary failure before provider invocation does not move a root
   ticket into `running`.
2. A root ticket with no call, no attempt, no admission, no mathematical node,
   no frontier event, and no attempt-ledger row is classified independently of
   aggregate `run_receipt.json`.
3. A second orphaned or pre-provider failure is detected, not hidden by the
   first recovered failure.
4. Readiness output is report-only and cannot mutate `expert-frontier-001`.
5. Runtime validation reports the nonterminal blocker with a stable typed code.
6. The run can be classified as either migratable or superseded without
   fabricating provider evidence.

### Allowed File Families

The repair may touch only the minimal runtime/readiness surfaces required by
the ticket, expected to be under:

- `labs/crouzeix_proof_reproduction/expert_runner.py`
- `labs/crouzeix_proof_reproduction/tickets.py`
- `labs/crouzeix_proof_reproduction/run_frontier.py`
- `labs/crouzeix_proof_reproduction/frontier_store.py`
- focused tests under `labs/crouzeix_proof_reproduction/tests/`
- the `CPFR-R014` tracker subtree and receipt evidence

If another file is required, the implementation must justify why the existing
runtime boundary cannot enforce the invariant.

Raw amendments identify `_run_expert_attempt()` in `expert_runner.py` as the
first repair seam because provider-boundary validation must happen before a
root ticket transitions to `running`.

### Forbidden Commands And Actions

The repair must not run live providers, continue roots, evaluate roots, or
mutate `expert-frontier-001` as a side effect of diagnosis.

Forbidden command classes:

- `run_frontier.py run`
- `run_frontier.py recover-roots` when it mutates the run
- `run_frontier.py evaluate-roots`
- `run_frontier.py generations`
- `run_frontier.py finalize`
- any `traecli exec` provider call for `expert-frontier-001`
- any network, MCP, or delegation action used to complete proof roots

## Trace Evidence Protocols

Agent traces are evidence, not authority. They may explain why the current
state exists; they cannot override fresh validators or run artifacts.

Before using a local trace as evidence, the agent must prove both:

```text
worktree: <harp-root>/.worktrees/crouzeix-proof-reproduction
run_id: expert-frontier-001
```

Trace files from other Harp worktrees may reveal repository hazards, but they
must not authorize CPFR-032 continuation, alter `CPFR-R014`, or replace fresh
validators on the active run.

Trace inspection must be metadata-first:

1. Read `session_meta` records from JSONL traces and filter by `cwd`.
2. Accept only traces whose `cwd` is the active proof-reproduction worktree or
   whose commands explicitly run from that worktree.
3. Require `expert-frontier-001` or one of its ticket IDs before treating a
   trace as run evidence.
4. Inspect named JSONL trace files before inspecting tool-result artifacts.
5. Avoid broad `rg` over `*.artifacts/tool-results/*.txt` unless a specific
   tool call ID is known and relevant.
6. Exclude the active review rollout from the candidate trace set.

Fresh validators outrank stale closeout text. A prior trace claim that there
were “No blockers” cannot authorize root continuation when current runtime
validation still fails on `expert-g0-operator-dilation`.

If runtime output, artifact inventory, bounded trace facts, source seam, and
amendment reachability state are unchanged, stop reviewing. The next action is
CPFR-R014 implementation or coherent guidance landing, not another
diagnosis-only amendment.

## Amendment Reachability Protocol

Before relying on tracker-linked amendment files, a proof agent must verify
that every active amendment named by the tracker exists in the commit or
staging surface being handed off.

An active tracker pointer to an untracked amendment file is not durable project
guidance. It may help inside a dirty worktree, but it will disappear from a
clean checkout or independent agent worktree unless captured in tracked files.

This capture and `agent-trace-index.md` replace the raw untracked amendment
files as the durable handoff surface.

## Verification Before Cleanup

Before deleting the raw amendment files from the old proof-reproduction
worktree:

1. Confirm this capture includes `CPFR-R014`, `expert-frontier-001`,
   `operator-dilation`, forbidden commands, trace protocols, and amendment
   reachability rules.
2. Confirm `agent-trace-index.md` lists amendments `005` through `012`.
3. Confirm `git diff --check` passes for the captured documents.
4. Confirm the captured documents are staged or committed in the integration
   worktree.
5. Remove only the old raw amendment files, not ignored `.runs` evidence.
