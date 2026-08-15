# Crouzeix Proof Reproduction Design

## Objective

Run a prospective, auditable attempt to reproduce a proof of Crouzeix's
conjecture from the public theorem statement and improve the proof-search
machinery exposed by the historical Jin prompt.

The study has two distinct goals:

1. test whether a pinned current model can produce a complete proof under a
   clean-room approximation of the historical task contract; and
2. test whether a DGM-style archive of dedicated expert agents makes the
   search more diverse, falsifiable, and reviewable than one monolithic root
   prompt.

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
budget, does a DGM-selected dedicated-expert frontier improve:

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

### Arm O0: Superseded flat orchestration baseline

`O0` turns the historical prompt's hidden control policy into a fixed sequence
of three independent route workers, one controller, an optional redirect,
synthesis, criticism, and repair.

The attempted `orchestrated-001` run is preserved as a superseded baseline. It
completed three independent route workers and one controller before the user
changed the treatment to dedicated expert agents with an explicit frontier.
Its redirect was terminated and the run was sealed `not_promoted`. No output
from this arm is silently imported into the replacement arm.

### Arm E: DGM-selected dedicated-expert frontier

`E` is the approved improved treatment. A fixed outer harness maintains an
archive of immutable expert nodes and launches each expert in a fresh
`traecli exec` session. Experts do not inherit a conversation and cannot
delegate further.

#### Expert node contract

Every expert receives one self-contained role packet:

- theorem bytes and leakage tier;
- expert role and bounded subtask;
- node ID, parent ID, generation, and selected direction ID;
- content-addressed parent artifacts explicitly allowed into context;
- forbidden sources and tools;
- success and functioning-node criteria;
- strict output schema; and
- an explicit prohibition on child delegation.

Every expert returns one node record:

- node and parent IDs;
- expert role and selected direction;
- self-classified proof family;
- concrete mechanism;
- proved intermediate statements;
- unproved obligations with stable IDs, strength, and recommended expert role;
- circularity risks;
- proposed next directions;
- candidate proof text or exact blocker; and
- confidence basis.

The result, prompt, context manifest, event stream, final response, and receipt
are persisted under `archive_nodes/<node_id>/`. Initial experts receive no peer
output. Descendant experts receive only their selected parent's immutable node
record and the one selected obligation or direction.

#### Initial expert roster

Generation zero launches five pristine root experts:

1. `function_theory`: analytic functional calculus, positive kernels, and
   boundary transforms;
2. `operator_dilation`: dilation, power families, recurrences, and operator
   inequalities;
3. `matrix_extremal`: finite-dimensional matrix, singular-vector, and extremal
   arguments;
4. `completion_positivity`: positive-real completion, interpolation, and
   matrix-valued positivity; and
5. `approximation_audit`: approximation, boundary regularity, limit order, and
   hidden-hypothesis audit.

These role names describe expertise, not known proof mechanisms. Their prompts
must not contain Jin- or Lorist-Schwenninger-specific answer vocabulary.

#### Independent proof-progress evaluation

Every schema-valid node is evaluated by two fresh dedicated evaluators with no
access to its treatment label, lineage score, or public proofs. They receive
the theorem, candidate node, and the same ten route-neutral probes:

1. objects, domains, and target are well-typed;
2. the implication chain is explicit;
3. a concrete nontrivial mechanism is present;
4. at least one intermediate lemma is actually proved;
5. hypotheses are tracked through use;
6. central identities or inequalities survive checking;
7. no known internal counterexample is exposed;
8. the route does not invoke the target or an equivalent statement
   circularly;
9. no theorem-strength unproved obligation remains; and
10. a standalone endpoint survives adversarial review.

A probe passes only when both evaluators return `pass`. Disagreement,
`insufficient_evidence`, or one failed evaluator does not pass. The node score
is:

$$
\alpha_i = \frac{\text{unanimously passed probes}}{10}.
$$

The scalar is a fixed search heuristic, not a calibrated probability of
correctness. Evaluator findings and disagreements remain attached to the node.

#### Functioning-node admission

A generated child is functioning when:

- the provider call and strict schema validation complete;
- the context manifest proves compliance with the declared leakage boundary;
- the result contains a concrete mechanism plus at least one proved statement,
  or a precise falsifiable blocker plus a materially new direction;
- all referenced parent and direction IDs exist; and
- no artifact, path, or size boundary is violated.

The archive uses DGM's `keep_all` rule: every functioning child is admitted,
even when its score is lower than its parent's. Admission means the node is a
usable search state, not that it is correct, improved, or promoted.

#### DGM parent selection

At generation \(t\), the archive is \(A_t\). Each node records score
\(\alpha_i\) and functioning-child count \(n_i\). Follow the DGM paper
contract, including the perfect-score eligibility rule:

$$
E_t = \{a_i \in A_t : \alpha_i < 1\}.
$$

For every eligible node:

$$
s_i = \frac{1}{1+\exp(-10(\alpha_i-0.5))},
\qquad
h_i = \frac{1}{1+n_i},
\qquad
w_i = s_i h_i,
\qquad
p_i = \frac{w_i}{\sum_j w_j}.
$$

The harness draws \(k=2\) parents with replacement per generation. The study
precommits `selection_seed=20260814`. To avoid runtime-library drift, each
uniform draw is derived from:

```text
SHA-256("crouzeix-frontier/v1\0" || seed || generation || draw_index)
```

interpreted as a big-endian integer divided by \(2^{256}\), then located in
the cumulative probability intervals sorted by node ID. Every selection event
records the eligible archive, scores, functioning-child counts, sigmoid terms,
underexploration terms, normalized probabilities, uniform draw, and selected
node.

This copies DGM's score-times-underexploration allocation. It does not relabel
child count as semantic novelty and does not use the released-code path that
omits the paper's perfect-score filter. The source contract is documented in
[DGM algorithm derivation](../../../knowledge/darwin_godel_machine/03_algorithm_derivation.md#eligibility)
and the pinned implementation at
[`DGM_outer.py:91`](../../../evidence/implementations/dgm/snapshot/DGM_outer.py#L91).

#### Child direction and expert role

For each selected parent, the deterministic frontier projection ranks open
directions by:

1. theorem-strength obligation;
2. evaluator critical finding;
3. major obligation;
4. evaluator major finding;
5. local obligation; and
6. parent-proposed materially new direction.

Ties use stable direction ID. The direction carries its recommended expert
role from the node or evaluator record; unknown roles fall back to
`approximation_audit`. Selecting the same parent twice in one generation
chooses its first and second open directions when available, otherwise it may
retry the same direction with distinct child IDs. The child prompt requires a
new mechanism or a rigorous closure of the selected obligation.

Only admitted functioning children increment the parent's \(n_i\), matching
DGM's functioning-child count rather than all attempted descendants.

#### Frontier loop and budget

The fixed outer harness runs:

- five generation-zero experts;
- two fresh evaluators per functioning node;
- at most three child generations;
- two sampled parent slots per generation;
- at most eleven functioning expert nodes;
- at most 33 expert/evaluator provider calls;
- one hour per provider call;
- a 4 GiB host free-space safety floor checked before every call; and
- no automatic retry under the same call or node ID.

The loop stops early when an archive node reaches \(\alpha_i=1\). Such a node
is frozen for separate reference-aware mathematical review; the search score
does not itself certify the proof. If no node reaches one, the run ends at the
fixed budget with the strongest nodes and exact frontier gaps.

#### Durable frontier state

The main orchestrator persists:

```text
attempt_ledger.jsonl
frontier_events.jsonl
selection_events.jsonl
archive_nodes/<node_id>/
frontier_snapshot.json
run_receipt.json
```

The three JSONL files are append-only. `frontier_snapshot.json` is a
deterministic projection and never edited as an authority. Previous failed,
blocked, superseded, and lower-scoring nodes remain in the archive.

### Arm G: Mechanism-guided diagnostic

`G` runs only after both blind candidates are sealed. It receives:

- the theorem statement;
- a concise, attributed mechanism card for either Jin's positive-real
  completion route or Lorist-Schwenninger's perturbation route; and
- the same standalone-proof output contract.

Its output is evaluated as reconstruction, not discovery.

## State Model

Expert nodes use this closed state set:

```text
attempted
functioning
archived
selected
complete
blocked
superseded
```

Allowed transitions are:

```text
attempted -> functioning
attempted -> blocked
functioning -> archived
archived -> selected
archived -> complete       only when alpha = 1
selected -> archived       after child attempt
archived -> superseded     only for treatment replacement, never deletion
```

Every transition is append-only and records sequence, run ID, node ID, parent
ID, generation, expert role, direction ID, prompt digest, output digest,
evaluation digests, score, reason, and timestamp.

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
4. the `H` and `O0` attempts are preserved and one `E` run is attempted and
   sealed;
5. failures, timeouts, or non-promotion are retained as first-class results;
6. blind outputs are reviewed only after sealing;
7. a guided run is attempted only if it has a distinct leakage label;
8. the reader packet reports results without overstating historical or
   mathematical reproduction;
9. all new documents and evidence are registered;
10. `mise run verify` passes; and
11. the verified commits are fast-forwarded locally without pushing.
