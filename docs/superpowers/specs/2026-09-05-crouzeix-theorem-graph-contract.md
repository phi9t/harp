# Crouzeix Theorem Graph Contract

## Objective

Adopt the useful part of the FLT/Prove2Me mechanism for Harp's Crouzeix proof
work: theorem statements become the stable collaboration objects, while proof
attempts, receipts, source locators, and read-backs attach to those statements.

The first implementation is local and read-only. It does not call Prove2Me,
launch missions, submit proofs, or change the canonical Crouzeix Markdown
packet.

## Inputs

The first slice consumes the three published route manifests:

- `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json`
- `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/route-manifest.json`
- `labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json`

Older route-local maps remain compatibility inputs for future checks:

- `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/source-map.json`
- `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json`

The route manifests are the first executable source because they already bind
node IDs, declaration names, module paths, statement hashes, source locators,
route dependencies, allowed axioms, reviews, and receipts.

## Output

`crouzeix-theorem-graph/v1` is a generated JSON object with:

- `schema_version`;
- `routes`, one summary per route manifest;
- `nodes`, one theorem card per unique `(declaration, statement_sha256)`;
- `frontier`, derived from node proof and audit status.

The graph is an inspection product, not source authority. Existing route
manifests and evidence receipts remain authoritative.

## Node Contract

Each graph node records:

- stable `node_id`;
- `route_ids` and `route_node_ids`;
- `roles`;
- `lean_name`;
- `statement_sha256`;
- `statement_text_paths`;
- optional `natural_language_statement`;
- `source_locators`;
- `dependency_ids`;
- `proof_status`;
- `proof_receipt_paths`;
- `allowed_axioms`;
- `readback_status`;
- `claim_ceiling`.

`natural_language_statement` starts as `null` until read-back receipts exist.
Lean statement text and `statement_sha256` are authoritative over prose.

## Validation Rules

The graph builder must reject:

1. duplicate graph node IDs;
2. dangling dependencies;
3. cycles or unstable topological ordering;
4. reused-route nodes without proof-bearing upstream route dependency evidence;
5. proof-bearing nodes without receipt paths and allowed axioms;
6. source-backed nodes without source locators;
7. incompatible declarations collapsed into one node.

The builder must keep route reuse visible. A Harp node reusing
Lorist-Schwenninger evidence may be `passed-local`, but it cannot become
independent evidence merely by appearing in a second route.

## Frontier Semantics

The first report exposes:

- `open_nodes`: nodes with no local or external proof status;
- `blocked_nodes`: nodes whose proof status is blocked;
- `missing_readbacks`: terminal, consequence, source-backed, and reused nodes
  without current read-back receipts;
- `external_candidates`: locally passed source-backed or derived nodes that are
  small enough to consider for a future Prove2Me pilot after human review.

This is deliberately conservative. A node should disappear from
`missing_readbacks` only after a separate read-back receipt is attached.

## Prove2Me Boundary

Prove2Me is an optional downstream collaboration surface. Before any Crouzeix
node is mirrored externally, Harp must have:

- a validated graph;
- a checked statement hash;
- a source locator or explicit `Harp-authored` status;
- current read-back;
- human approval to use Prove2Me.

Imported Prove2Me results must return as evidence with theorem ID, environment,
formal statement, statement hash, proof hash, dependency graph, verdict, local
Lean rerun, axiom audit, and claim-ceiling update.
