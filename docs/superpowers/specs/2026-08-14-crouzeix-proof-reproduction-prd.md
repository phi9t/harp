# Crouzeix Proof Reproduction PRD

## Objective

Add a self-contained Harp experiment that:

1. executes the recoverable historical Crouzeix root prompt under a pinned,
   recorded current runtime;
2. executes an improved externally orchestrated proof search under matched
   access and model controls;
3. preserves every call, route decision, candidate, critic finding, and
   promotion decision;
4. separates blind generation from reference-aware verification; and
5. publishes a source-backed account of what was reproduced, blocked, or
   disproved.

The approved experimental contract is
[`2026-08-14-crouzeix-proof-reproduction-design.md`](2026-08-14-crouzeix-proof-reproduction-design.md).
This PRD fixes the product and ownership boundaries needed to implement it.

## Problem

The public Jin prompt contains useful search policy:

- preserve independent approaches;
- maintain a proof-family registry;
- block theorem-strength reductions;
- reopen only for a new mechanism;
- use adversarial agents throughout; and
- continue until a complete proof survives audit.

Those controls are instructions inside one prompt. The public corpus contains
no run record showing that they happened. It also lacks the exact runtime,
worker prompts, budgets, route transitions, failed candidates, critic
findings, and selection lineage.

Running the same string again would therefore produce another unauditable
outcome. The missing product is a proof-search harness that makes those control
states observable without leaking either known proof into blind generation.

## Users

### Research operator

Runs one prospective study, monitors bounded provider calls, seals results, and
can explain exactly what each arm saw.

### Mathematical reviewer

Receives an anonymized frozen candidate and returns locator-based findings
without seeing the treatment label or model cost.

### Harp reader

Needs a concise result that distinguishes:

- historical prompt recovery;
- prospective execution;
- mathematical completeness;
- mechanism similarity;
- formal-verification status; and
- unresolved contamination or process gaps.

## Product Surface

### Lab

`labs/crouzeix_proof_reproduction/` is the reusable operator surface.

It provides:

```text
prepare_run.py
run_experiment.py
seal_run.py
protocol.py
prompts/
schemas/
tests/
README.md
```

The lab uses the Python standard library and calls TRAE CLI through a typed,
content-addressed boundary. It does not depend on another checkout.

### Mutable run state

`labs/crouzeix_proof_reproduction/.runs/` is ignored. Each run directory is
create-only and contains:

```text
run_spec.json
inputs/
calls/
route_events.jsonl
candidate/
review/
run_receipt.json
```

The historical prompt is supplied by the operator and must match its pinned
byte count and digest. Harp does not vendor those upstream bytes.

### Published evidence

One selected study is published under:

`evidence/crouzeix_conjecture_reproduction/`

It contains:

- a deterministic raw run archive;
- a raw-member digest inventory;
- normalized prompts authored by Harp;
- normalized candidates and reviews;
- per-call and aggregate receipts;
- route-state events;
- a manifest;
- a provenance and rights record; and
- no rewritten upstream prompt or manuscript bytes.

The original historical prompt remains remote-only and is represented by its
existing source receipt plus the run's input digest.

### Canonical knowledge

`knowledge/crouzeix_conjecture/` gains:

- the bounded reproduction research note;
- an experiment protocol and prompt-machinery explanation; and
- a result report with candidate and review dispositions.

Material claims extend the existing `CC-*` claim ledger and source registry.
The packet index links every new document.

## Functional Requirements

### FR1: Historical input verification

The preparation command rejects a historical prompt unless:

- it is a regular non-symlinked file;
- it is exactly 4,106 bytes; and
- its SHA-256 is
  `0a0c3000b81efc4d9edc65ec3cd1d53df0d4e69b24bfee9fe0860301d853d6fc`.

The command records the original digest, applies only the approved output-path
normalization, and records the resulting execution digest and byte-level
normalization description.

### FR2: Strict run specification

Preparation writes a strict `run_spec.json` with no unknown fields. It binds:

- arm and leakage tier;
- model and CLI identity;
- access policy;
- tools;
- maximum calls and timeout;
- prompt and schema digests;
- input file inventory; and
- generation-excluded reference classes.

Invalid or unknown enum values fail before any provider call.

### FR3: Create-only provider calls

Every provider call:

- runs in a fresh empty workspace;
- uses `--ignore-user-config`, `--ignore-rules`, and `--ephemeral`;
- pins model, sandbox, approval, and network policy;
- writes a complete call receipt;
- refuses an existing call directory;
- retains nonzero exits, malformed outputs, and timeouts; and
- never retries silently.

### FR4: Historical arm

The historical arm executes exactly one root call. The normalized prompt asks
for `candidate.tex` in the call workspace. The harness accepts only:

- a regular non-symlinked `candidate.tex`; or
- an explicitly recorded missing-candidate outcome.

A final chat response is evidence of the call, not an automatic substitute for
the requested file.

### FR5: Observable orchestration arm

The improved arm implements:

1. three independent route calls;
2. one controller call;
3. zero or one redirect call when family collapse is recorded;
4. one synthesis call;
5. two independent critic calls;
6. one repair call; and
7. one deterministic promotion decision.

Each phase has a closed JSON schema. No worker sees peer output before all
initial route records are sealed.

### FR6: Route-state enforcement

The protocol validates the closed route state machine from the approved
design. It rejects:

- duplicate route IDs;
- invalid transitions;
- reopening without a new mechanism;
- viability without concrete mathematical artifacts;
- audit without a frozen candidate digest;
- promotion with an unresolved critical finding; and
- promotion with a theorem-strength unproved obligation.

### FR7: Budget and usage accounting

The run receipt reports:

- call count;
- successful, failed, and timed-out calls;
- root/controller/worker/critic/repair token subtotals;
- aggregate input, cached input, output, and reasoning tokens;
- start and completion times; and
- whether the configured call and timeout ceilings were reached.

Provider absence of a metric is explicit `null`, not zero.

### FR8: Deterministic sealing

The seal command:

- validates every run artifact;
- scans for credential patterns;
- normalizes only Harp-authored JSON and Markdown;
- creates a deterministic gzip tar archive;
- writes member and normalized manifests;
- refuses an existing evidence destination; and
- produces a receipt binding the archive and selected candidate digest.

### FR9: Reference-aware review

Review happens only after candidate sealing. Review records:

- anonymous candidate ID and digest;
- completeness outcome;
- finding IDs, severity, locator, statement, and disposition;
- theorem-strength obligations;
- Jin-like and Lorist-Schwenninger-like mechanism similarity;
- formalization status; and
- reviewer/model identity and access boundary.

Mechanism similarity cannot change the completeness outcome.

### FR10: Reader integration

The resulting packet must answer:

- what historical machinery is actually recoverable;
- what was changed to execute it;
- what the improved controller adds;
- which arms were attempted;
- what candidates were produced;
- which mathematical defects were found;
- whether any candidate was promoted;
- what formal verification was or was not attempted; and
- what claims remain impossible due to contamination and missing history.

## Non-Goals

- Reconstruct hidden chain-of-thought.
- Claim the exact historical model or private runtime was replayed.
- Vendor the unlicensed historical prompt, manuscripts, or Lean tree.
- Treat model self-evaluation as mathematical certification.
- Treat a proof's resemblance to Jin or Lorist-Schwenninger as correctness.
- Automatically submit, announce, or publish a conjecture resolution.
- Make live provider calls part of `mise run verify`.
- Generalize the first implementation into a theorem-proving framework.
- Add a database or service.

## Strict Types

The Python implementation uses validated dictionaries at JSON boundaries, with
closed string enums for:

```text
arm: historical | orchestrated | guided
leakage: L0 | L1 | L2 | L3 | L4
route_state: independent | blocked | viable | audited | promoted | rejected
call_role: historical_root | route_worker | controller | redirect |
           synthesizer | logical_critic | operator_critic | repair
call_status: completed | failed | timed_out | malformed
finding_severity: critical | major | minor
finding_disposition: fixed | rejected_with_reason | unresolved
proof_outcome: complete | incomplete | invalid | indeterminate
formal_status: not_attempted | blocked | failed | passed
```

Unknown fields and values are rejected.

## Simplicity Gate

### What caller knowledge does this delete?

Operators no longer hand-compose CLI commands, infer whether independent
rounds occurred, inspect event logs manually, or decide promotion from prose.

### Which interface shrinks?

The operator interface becomes:

```sh
python3 prepare_run.py ...
python3 run_experiment.py RUN_DIR
python3 seal_run.py RUN_DIR ...
```

### Where does locality improve?

CLI invocation, output validation, route transitions, accounting, and sealing
live in one lab. Mathematical conclusions remain in the packet.

### What authority does this avoid duplicating?

The harness records and validates experiment state. It does not become a
mathematical oracle, corpus authority, or proof source.

### Where does complexity return if the lab is deleted?

It returns to one-off shell commands, private transcripts, manually inferred
route states, and result prose that cannot be independently recomputed.

## Acceptance

- All functional requirements have deterministic fake-provider tests.
- The tests demonstrate red-green behavior for production changes.
- `mise run verify` remains provider-neutral.
- One historical and one orchestrated live run are attempted.
- Every attempt is retained, including failures and non-promotion.
- Published evidence passes strict offline verification.
- The packet and generated Atlas/search outputs are current.
- The final payload receipt is refreshed only after all tracked content settles.
- The branch is adversarially reviewed, fully verified, and landed locally
  without pushing.
