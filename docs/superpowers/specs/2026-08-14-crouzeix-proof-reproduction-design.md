# Crouzeix Proof Reproduction Design

## Objective

Run a prospective, auditable attempt to reproduce a proof of Crouzeix's
conjecture from the public theorem statement and improve the proof-search
machinery exposed by the historical Jin prompt.

The study has two distinct goals:

1. test whether a pinned current model can produce a complete proof under a
   clean-room approximation of the historical task contract; and
2. test whether an external controller makes the search more diverse,
   falsifiable, and reviewable than one monolithic root prompt.

The study does not claim to replay Jin's private run. The public corpus lacks
its exact runtime, worker prompts, transcripts, route registry, budgets,
selection lineage, and edit history. The historical root prompt is an input
anchor, not a complete experimental record.

## Research Questions

### RQ1: Historical-prompt reproduction

Can `gpt-5.6-sol`, invoked through the pinned local TRAE CLI version, produce a
complete proof when given the historical 4,106-byte root prompt in an empty,
network-disabled workspace?

The execution prompt differs from the historical blob only by replacing its
machine-specific absolute output path with `candidate.tex`. Both the original
and execution prompt digests must be retained.

### RQ2: Machinery improvement

Under the same model, theorem statement, access policy, and declared aggregate
budget, does an external controller improve:

- independent proof-family coverage;
- visibility of theorem-strength blockers;
- critic defect yield;
- disposition of rejected routes;
- provenance completeness; and
- the mathematical quality of the promoted candidate?

### RQ3: Diagnostic completion

If blind runs fail, can a clearly labeled mechanism-guided arm reconstruct a
complete Jin-like or Lorist-Schwenninger-like route?

This arm diagnoses synthesis and proof-writing ability. It cannot support an
independent-rediscovery claim.

## Evidence Boundary

### Generation-visible inputs

Blind generation may see only:

- the theorem statement;
- its assigned prompt treatment;
- outputs explicitly passed by the controller from earlier blind phases; and
- an empty run workspace.

Blind generation may not see:

- either public proof;
- Jin's AI disclosure;
- Lorist-Schwenninger's preprint;
- Lean declaration names or source;
- Harp's Crouzeix packet;
- evaluator rubrics containing proof-specific checkpoints;
- Git history beyond the prompt bytes; or
- the internet.

### Verification-visible inputs

After a candidate is frozen and content-addressed, evaluators may inspect:

- the candidate;
- Jin's formalization-matched v4;
- Lorist-Schwenninger arXiv v1;
- Harp's proof reconstructions and critical review;
- the pinned Lean source and audit map; and
- run receipts and route-state events.

Verification evidence must not flow back into a blind generation run.

### Authority separation

- `docs/superpowers/` owns plans and intent.
- `labs/crouzeix_proof_reproduction/` owns executable experiment machinery.
- ignored `.runs/` directories own mutable attempts.
- `evidence/crouzeix_conjecture_reproduction/` owns sealed run bytes and
  normalized receipts.
- `knowledge/crouzeix_conjecture/` owns technical synthesis and conclusions.
- the claim ledger owns material reader-facing claims.

A prompt, transcript, issue, commit, or private memory is not the authority for
the experiment's intent or mathematical conclusion.

## Experimental Arms

### Arm H: Historical root contract

`H` is the closest executable approximation of the recoverable public task.

- Input: historical root prompt at Git blob
  `5b2705db56787157fadbfd9416522feb69b4ad95`.
- Original SHA-256:
  `0a0c3000b81efc4d9edc65ec3cd1d53df0d4e69b24bfee9fe0860301d853d6fc`.
- Allowed normalization: replace the absolute output directory in line 30
  with `candidate.tex`.
- Runtime: one root TRAE CLI invocation.
- Model: `gpt-5.6-sol`.
- Network: disabled.
- Workspace: empty except schema and output targets owned by the harness.
- Result: one candidate proof plus the raw CLI event stream.

This arm measures the root prompt as a monolithic executable contract. It
cannot establish whether its internal request for multi-agent behavior was
honored unless the event stream exposes such launches.

### Arm O: Observable external orchestration

`O` turns the historical prompt's hidden control policy into explicit phases.

#### Phase 1: Independent routes

Launch three workers with identical theorem bytes and no peer output. Each must
return one route record:

- route ID;
- self-classified proof family;
- concrete mechanism;
- proved intermediate statements;
- unproved obligations;
- circularity risks;
- candidate proof text or exact blocker; and
- confidence basis.

The harness must not tell workers the favored route or proof-specific
mechanism vocabulary.

#### Phase 2: Registry and redirect

A controller receives only the sealed route records. It:

- merges superficial duplicates into family labels;
- marks routes `blocked` or `viable`;
- identifies theorem-strength obligations;
- selects at most two routes for continuation; and
- emits one underexplored search brief if the three workers collapsed onto one
  family.

A blocked route can reopen only with a materially new mechanism recorded in
the event log.

#### Phase 3: Synthesis

A synthesizer receives viable route records and controller decisions. It
returns one standalone candidate and an obligation ledger. It may combine
routes only after their independent records are sealed.

#### Phase 4: Adversarial review

Two critics independently inspect the frozen candidate:

- a logical critic checks unsupported implications, circularity, quantifiers,
  limit order, and theorem-strength hidden lemmas;
- an operator-theory critic checks domain hypotheses, adjoints,
  commutativity, boundedness, positivity, and norm estimates.

Critics return findings with stable IDs, severity, exact candidate locator,
and a falsifying example or missing proof obligation when available.

#### Phase 5: Repair and promotion

One repair pass receives the candidate and both finding sets. It must map each
finding to `fixed`, `rejected-with-reason`, or `unresolved`.

Promotion requires:

- a standalone candidate;
- no unresolved critical finding;
- no theorem-strength missing lemma;
- a complete obligation ledger; and
- a sealed candidate digest.

The harness records `not_promoted` when these conditions do not hold. It does
not instruct a model to call an incomplete candidate complete.

### Arm G: Mechanism-guided diagnostic

`G` runs only after both blind candidates are sealed. It receives:

- the theorem statement;
- a concise, attributed mechanism card for either Jin's positive-real
  completion route or Lorist-Schwenninger's perturbation route; and
- the same standalone-proof output contract.

Its output is evaluated as reconstruction, not discovery.

## State Model

Routes use this closed state set:

```text
independent
blocked
viable
audited
promoted
rejected
```

Allowed transitions are:

```text
independent -> blocked
independent -> viable
blocked -> independent     only with a new mechanism receipt
viable -> audited
audited -> promoted        only with zero unresolved critical findings
audited -> rejected
```

Every transition is append-only and records sequence, run ID, route ID,
parent route IDs, prompt digest, output digest, reason, and timestamp.

## Strict Artifacts

### Run specification

`run_spec.json` is written before execution and contains:

- schema version;
- run and arm IDs;
- leakage tier;
- CLI path, version, and executable digest;
- model string;
- original and execution prompt digests;
- exact prompt normalization;
- sandbox and approval policy;
- allowed tools;
- network policy;
- timeout;
- maximum calls;
- aggregate token accounting boundary;
- generation-visible file inventory; and
- reference/evaluator material excluded from generation.

### Provider call receipt

Each call directory contains:

```text
request.json
prompt.md
output.schema.json
events.jsonl
stderr.txt
final.json
receipt.json
```

`receipt.json` binds command argv, CLI identity, prompt, schema, final output,
events, stderr, timestamps, return code, session ID when exposed, and token
usage. Existing call directories are create-only.

### Route registry

`route_events.jsonl` contains strict state-transition objects. The current
registry is a deterministic projection, never an independently edited file.

### Candidate and review

The final run directory contains:

```text
candidate.tex
candidate.sha256
obligations.json
critic_findings.json
promotion.json
run_receipt.json
```

The candidate is frozen before critics run. A repaired candidate is a new
version with a new digest and explicit parent digest.

## Prompt Design

The improved prompts must improve machinery, not leak the answer.

### Root principles

1. State the access and success boundary first.
2. Require structured outputs instead of relying on prose compliance.
3. Separate discovery, synthesis, criticism, and repair into different calls.
4. Preserve independent contexts until route records are sealed.
5. Demand concrete equations or exact blockers from every worker.
6. Treat theorem-strength obligations as blockers, not progress.
7. Require critics to attempt falsification rather than provide reassurance.
8. Permit `not_promoted` as a valid outcome.
9. Bind every accepted transfer to parent route IDs.
10. Keep proof-specific evaluator checkpoints out of blind prompts.

### Historical versus improved treatment

The historical arm keeps the public root task wording except for its output
path. The improved arm uses the same theorem and completeness criterion but
moves orchestration into the harness and uses phase-specific strict schemas.
This means the experiment compares complete systems, not two strings with an
uncontrolled runtime.

## Evaluation Contract

### Primary endpoint

`complete-proof` means an independent reviewer finds a complete implication
chain from stated hypotheses to the constant-two conclusion with:

- no circular use of Crouzeix's conjecture or an equivalent theorem;
- no unproved theorem-strength lemma;
- all operator domains and adjoints well-typed;
- every positivity, commutativity, boundedness, and limiting step justified;
- no reliance on fixed-dimensional computation as proof; and
- a clear reduction from the stated matrix-polynomial theorem to the proved
  intermediate result.

The allowed outcomes are:

```text
complete
incomplete
invalid
indeterminate
```

`indeterminate` is used when review or formal verification is blocked.

### Secondary endpoints

- audit survival;
- unproved-obligation count by severity;
- independent-family coverage before cross-pollination;
- critic defect yield and closure;
- tokens and wall time to first viable mechanism;
- provenance completeness;
- Jin-mechanism similarity;
- Lorist-Schwenninger-mechanism similarity; and
- formal verification status.

Mechanism similarity is diagnostic and never substitutes for correctness.

### Evaluator independence

Candidates are anonymized before mathematical review. Reviewers do not see:

- arm label;
- model or token cost;
- controller decisions;
- whether a candidate resembles either public proof; or
- the other candidate.

One separate classifier may compare the reviewed candidate to public
mechanisms after correctness findings are sealed.

### Formalization status

Use the closed status set:

```text
not_attempted
blocked
failed
passed
```

The existing Jin builds remain `blocked`. A generated candidate is not Lean
verified merely because it resembles a formalized route.

## Safety And Reproducibility

- The harness uses Python standard library only.
- Model invocations run in empty per-call workspaces.
- `--ignore-user-config`, `--ignore-rules`, and `--ephemeral` are mandatory.
- Blind calls use no web search, MCP, or repository access.
- Secrets are scanned before publishing raw archives.
- Paths are normalized, relative, and symlink-free.
- Provider output and event logs have byte caps.
- Existing run directories and evidence destinations are never overwritten.
- Raw archives are deterministic and content-addressed.
- Tests use fake providers; `mise run verify` never launches a real model.
- Live execution is an explicit operator command, outside the release gate.

## Threats And Claim Ceiling

The study cannot eliminate training contamination. `gpt-5.6-sol` may already
know public 2026 proof artifacts. Therefore:

- supplied-context leakage is measured;
- training contamination is reported as unknown;
- a matching mechanism in a blind run is evidence of output under a clean
  context, not proof of first discovery;
- no result is called an exact reproduction of Jin's private run; and
- a successful generated proof still requires independent mathematical review.

The strongest possible conclusion is:

> Under the recorded current model, CLI, access policy, prompt treatment, and
> budget, the run produced a candidate with the recorded review disposition.

It is not:

> The historical AI discovery was replayed, or Crouzeix's conjecture is
> externally certified as settled.

## Acceptance Criteria

The work is complete when:

1. the bounded research note is registered and source-backed;
2. the harness validates strict run specifications and outputs;
3. fake-provider tests prove create-only calls, prompt isolation, event
   accounting, state transitions, and archive determinism;
4. at least one `H` run and one `O` run are attempted and sealed;
5. failures, timeouts, or non-promotion are retained as first-class results;
6. blind outputs are reviewed only after sealing;
7. a guided run is attempted only if it has a distinct leakage label;
8. the reader packet reports results without overstating historical or
   mathematical reproduction;
9. all new documents and evidence are registered;
10. `mise run verify` passes; and
11. the verified commits are fast-forwarded locally without pushing.
