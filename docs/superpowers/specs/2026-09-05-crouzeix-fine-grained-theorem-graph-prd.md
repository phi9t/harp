# Crouzeix Fine-Grained Theorem Graph PRD

## Problem Statement

The current Crouzeix theorem graph proves that the Jin, Lorist--Schwenninger,
and Harp routes are complete at the route-manifest level, but several graph
nodes are still too coarse for theorem-card collaboration.

The graph answers "does each route have a validated proof chain?" It does not
yet answer "which small theorem statement should a proof agent, reviewer, or
external prover work on next?" In particular, high-fan-in nodes such as the
Harp finite atomic L2 witness, the Harp terminal theorem, and Jin fixed
outer-domain convergence hide multiple independent mathematical obligations
behind one node.

This is acceptable for local certification, but weak for Prove2Me-style
collaboration. A collaboration surface needs smaller stable theorem cards,
explicit parent-child grouping, precise statement identity, clear proof
readbacks, and receipts that can be checked independently without flattening
the graph into every transitive Lean declaration.

## Solution

Add a second graph layer: a proof-obligation graph under selected route-level
nodes.

The existing `crouzeix-theorem-graph/v1` remains the stable route graph. It
continues to collapse route manifests by `(Lean declaration, statement hash)`
and remains the high-level inspection and route-completion product.

The new layer records optional child obligations for compound nodes. Each child
obligation is a theorem-card-sized unit with one formal statement, one natural
language readback, one parent route graph node, one proof target or declaration
receipt, and explicit dependencies on other obligation nodes or route nodes.

The design deliberately stops short of a raw Lean dependency graph. A node
should be split only when it becomes an independently assignable or reviewable
mathematical claim.

## User Stories

1. As a Harp proof engineer, I want to see which route nodes are too coarse, so
   that I can choose the next useful formalization split.
2. As a Harp proof engineer, I want one stable theorem card per proof
   obligation, so that agent work can be assigned without handing over an
   entire route.
3. As a reviewer, I want every fine-grained node to bind a Lean statement hash,
   so that prose cannot silently drift from formal content.
4. As a reviewer, I want parent route nodes to remain visible, so that route
   certification does not get lost inside low-level implementation detail.
5. As a Prove2Me collaborator, I want exported theorem cards to have clear
   prerequisites and readbacks, so that external attempts can target bounded
   statements.
6. As a Harp maintainer, I want imported external proof results to attach below
   existing graph nodes, so that external evidence does not become route
   authority until locally rerun and audited.
7. As a proof agent, I want a node frontier that names open, blocked, stale,
   and oversized obligations, so that the next action is mechanically obvious.
8. As a reader, I want the graph to show mathematical buildup rather than only
   final theorems, so that the Crouzeix proof routes are navigable.
9. As a Harp maintainer, I want route-level completion to remain independent of
   optional subgraph expansion, so that new graph detail does not regress
   existing proof completion.
10. As a future proof engineer, I want the graph split criteria written down,
    so that new nodes are added consistently instead of according to whatever
    one session happened to notice.

## Implementation Decisions

### Layered graph model

Keep two distinct graph products:

- `crouzeix-theorem-graph/v1`: route-level graph, already implemented.
- `crouzeix-proof-obligation-graph/v1`: fine-grained child graph for selected
  compound nodes.

The route graph remains the public route-completion summary. The obligation
graph is a collaboration and proof-planning surface.

### Node granularity rule

Split a route node only when the child claim is independently useful.

A child node is appropriate when at least one condition holds:

- it can be assigned to a proof agent without also assigning the full parent;
- it has a stable Lean declaration or should be promoted to one;
- it exposes a reusable lemma needed by multiple parents;
- it separates mathematical mechanisms that may fail independently;
- it is a natural Prove2Me theorem-card export;
- it isolates a source-faithfulness question from a local adaptation question.

A child node is not appropriate when it is only local proof plumbing, short
rewriting, a private helper whose statement is unstable, or a transitive import
edge with no independent mathematical meaning.

### Schema shape

The new graph should be generated from a versioned ledger, not hand-assembled
inside Python code. The first ledger should live beside the existing route
manifests:

```text
labs/crouzeix_proof_reproduction/formal_targets/proof-obligations.json
```

Each obligation entry should contain:

```json
{
  "node_id": "harp-l2-witness-boundary-compression-moments",
  "parent_node_id": "harp-counting-l2-witness-dimension-bound",
  "route_ids": ["harp"],
  "kind": "derived-obligation",
  "role": "intermediate",
  "lean_name": "CrouzeixConjecture.Harp.somePromotedDeclaration",
  "statement_sha256": "64 lowercase hex characters",
  "statement_text_paths": [
    "labs/crouzeix_proof_reproduction/formal_targets/harp/declaration-types/..."
  ],
  "dependency_ids": [
    "harp-reuse-boundary-embedding",
    "harp-reuse-boundary-multiplier"
  ],
  "proof_status": "passed-local",
  "proof_receipt_paths": [
    "evidence/crouzeix_conjecture/routes/harp/receipt.json"
  ],
  "readback_status": "current",
  "natural_language_statement": "..."
}
```

The first implementation may permit `lean_name: null` only for planned
obligations, but such nodes must have `proof_status: open` and must never count
as proof-complete. Passed nodes must bind an actual Lean declaration and a
statement hash.

### Parent-child contract

Every obligation node must name exactly one `parent_node_id` from the route
graph. Cross-parent dependencies are allowed only when they point to stable
route nodes or to other obligation nodes that are already bound to Lean
statements.

Parent route nodes keep their existing proof status. Child obligations add
planning and review detail; they do not weaken or replace the parent proof
receipt.

If all child obligations under a parent are passed and current, the parent may
report `obligation_status: expanded-current`. If any child is open, blocked,
stale, or missing a readback, the parent reports `expanded-incomplete`.

### Readback contract

Reuse the current readback binding discipline:

- current readbacks must bind node ID, Lean name, statement hash, and statement
  text path set;
- stale bindings must fail validation rather than silently downgrade;
- prose remains audit data, not proof authority.

For obligations, readbacks are mandatory for:

- terminal obligations;
- exported Prove2Me candidates;
- reused obligations;
- any obligation with source locators;
- any obligation whose parent is terminal or consequence-level.

### Frontier contract

Add frontier buckets for the obligation graph:

- `oversized_parent_nodes`: route nodes whose fan-in or statement complexity
  exceeds the split threshold and lack a current obligation expansion;
- `open_obligations`: planned obligations without passed local evidence;
- `blocked_obligations`: obligations with a known blocked precondition;
- `missing_obligation_readbacks`: proof-bearing obligations lacking current
  readbacks;
- `prove2me_candidates`: small, statement-bound, readback-current obligations
  that could be exported after explicit human approval.

The first split threshold should be conservative:

- parent route graph fan-in greater than or equal to 4;
- or terminal/consequence parent with more than one independent mathematical
  bridge;
- or reviewer-designated compound theorem.

### Prove2Me boundary

Do not add live Prove2Me API calls in this work.

The output should be ready for a later adapter by producing theorem-card
records with:

- theorem ID;
- Lean declaration name;
- statement hash;
- statement text path;
- dependencies;
- natural language readback;
- claim ceiling;
- local proof status;
- allowed axioms;
- source locator or Harp-authored status.

A later Prove2Me integration may consume this export only after explicit
approval. Imported Prove2Me results must return as evidence and must be rerun
locally before changing a Harp claim ceiling.

## Initial Crouzeix Split Plan

### Wave 1: Harp route

Start with Harp because it is the locally derived route and has the most hidden
fan-in.

Split `harp-counting-l2-witness-dimension-bound` into:

- `harp-l2-witness-cubature-input`: packages finite positive cubature moments.
- `harp-l2-witness-companion-commutation`: exposes companion-algebra
  compatibility.
- `harp-l2-witness-power-cauchy-moments`: binds polynomial power Cauchy moment
  identities.
- `harp-l2-witness-boundary-embedding-data`: assembles square root, embedding,
  and multiplier data.
- `harp-l2-witness-compression-moments`: connects L2 compression moments to
  finite matrix moments.
- `harp-l2-witness-dimension-count`: records the finite atomic counting and
  dimension bound.
- `harp-l2-witness-package`: produces `finiteAtomicL2DilationWitness`.

Split `harp-operator-recurrence` into:

- `harp-recurrence-finite-scalar-bound`: imports the finite weighted
  recurrence conclusion into Harp's finite-horizon setup.
- `harp-recurrence-completed-square-input`: packages the completed-square lower
  bound for the finite-horizon data.
- `harp-recurrence-operator-lower-bound`: derives the equation-three finite
  lower bound.

Split `harp-perturbation-endpoint` into:

- `harp-endpoint-norm-attainment`: binds the norm-attaining vector setup.
- `harp-endpoint-scalar-contradiction`: applies the scalar endpoint lemma.
- `harp-endpoint-target-norm`: proves the Harp target operator has norm at most
  two.

Split `harp-terminal-theorem` into:

- `harp-terminal-witness-to-finite-bound`: converts the L2 witness and
  perturbation endpoint into the finite-horizon bound.
- `harp-terminal-inner-limit-transfer`: applies the inner limiting lemma.
- `harp-terminal-outer-limit-transfer`: applies the outer approximation limit.
- `harp-terminal-main-theorem`: binds the final Harp finite-dimensional theorem.

### Wave 2: Jin route

Split `jin-fixed-outer-domain-convergence` into:

- `jin-outer-simple-spectrum-approximation`: simple-spectrum approximation
  input.
- `jin-outer-numerical-range-control`: numerical-range convexity and containment
  input.
- `jin-outer-function-maximum-limit`: maximum-modulus convergence input.
- `jin-outer-double-layer-bound-transfer`: applies the double-layer realization
  under fixed-domain hypotheses.
- `jin-outer-holomorphic-bound`: proves `holomorphicCrouzeixBound`.

Split `jin-positive-real-completion` only if future work needs external
collaboration below the current three-node completion cluster. Its current
dependencies already separate sampling, Gramian transfer, and eigenvector
endpoint obligations well enough for the next pass.

### Wave 3: Lorist--Schwenninger route

Split `ls-perturbation-lemma` into:

- `ls-perturbation-power-recurrence-input`: receives equation-three lower
  bound.
- `ls-perturbation-scalar-endpoint-input`: receives scalar contradiction.
- `ls-perturbation-norm-target`: proves `norm_target_le_two`.

Split `ls-double-layer-realization` only if Prove2Me collaboration targets the
boundary Cauchy and parametric realization layer directly. Otherwise keep it as
one route node because its current fan-in is one and it has source-local
structure.

## Implementation Plan

1. Add the `proof-obligations.json` ledger schema and parser.
2. Extend `theorem_graph.py` with a builder for
   `crouzeix-proof-obligation-graph/v1`.
3. Add validation for parent existence, acyclicity across route and obligation
   nodes, statement-hash binding, readback binding, proof-status consistency,
   and frontier recomputation.
4. Add a CLI subcommand to `proof_evidence.py` that prints the obligation
   graph and a compact human summary.
5. Seed Wave 1 with Harp obligation nodes. Mark genuinely statement-bound
   obligations as `passed-local`; mark proposed but not yet promoted Lean
   declarations as `open`.
6. Add tests that reject unknown parents, cycles, passed obligations without
   Lean statements, stale readbacks, and frontier mismatches.
7. Add a Prove2Me export dry-run command that writes JSON to stdout only. It
   must not call a network API.
8. Refresh derived repository receipts after the tracked payload settles.

## Testing Decisions

The highest useful seam is the graph builder and validator, because downstream
CLI output, Prove2Me dry-run exports, and reader displays should all consume
the same normalized object.

Tests should cover:

- building a valid obligation graph from the seeded ledger;
- accepting dependencies on route graph nodes and obligation graph nodes;
- rejecting unknown parent route nodes;
- rejecting dangling obligation dependencies;
- rejecting cycles among obligation nodes;
- rejecting `passed-local` obligations without Lean statement hashes;
- rejecting stale readback bindings;
- recomputing frontier buckets rather than trusting serialized frontier data;
- preserving current route graph completion when obligation children are open;
- producing deterministic Prove2Me dry-run exports.

Focused commands while iterating:

```sh
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_theorem_graph
python3 labs/crouzeix_proof_reproduction/proof_evidence.py theorem-graph
python3 labs/crouzeix_proof_reproduction/proof_evidence.py proof-obligation-graph
python3 labs/crouzeix_proof_reproduction/proof_evidence.py prove2me-export --dry-run
```

Release gate:

```sh
mise run verify
```

Do not run Lean cache hydration or update commands during this work.

## Acceptance Criteria

The work is complete when:

1. The existing theorem graph still validates with no open nodes, no blocked
   nodes, and no missing required readbacks.
2. A new obligation graph validates and reports Harp compound-node expansion
   status.
3. Wave 1 Harp obligations are present and deterministic.
4. Every `passed-local` obligation binds a Lean declaration, statement hash,
   statement text path, receipt path, and allowed axiom set.
5. Every exported Prove2Me candidate has a current readback and a checked
   statement hash.
6. Planned obligations that do not yet have promoted Lean declarations remain
   `open` and do not count as proof-complete.
7. Tests cover valid and invalid graph cases.
8. `mise run verify` passes after the import receipt digest is refreshed.

## Out of Scope

- Live Prove2Me setup, login, API calls, mission launch, or proof submission.
- Replacing route manifests as proof authority.
- Claiming external proof completion from readbacks or graph shape.
- Flattening the graph into every local Lean helper declaration.
- Rewriting captured evidence under `evidence/*/artifacts/`.
- Adding Markdown under `content/`.
- Hydrating or updating Lean dependencies.

## Further Notes

The first implementation should bias toward fewer, higher-quality obligation
nodes. The graph can always be deepened later, but every new node becomes a
maintenance obligation: it needs a stable identity, validation rules, readback
discipline, and a clear role in proof collaboration.

The practical target is not "more nodes." The target is a graph where each
open node is small enough for an agent or external collaborator to receive as a
self-contained theorem-card task, and where each completed node has enough
evidence that Harp can decide whether to trust, review, rerun, or export it.
