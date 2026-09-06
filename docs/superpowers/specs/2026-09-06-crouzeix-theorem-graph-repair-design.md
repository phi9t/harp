# Crouzeix Theorem-Graph Repair Design

**Status:** implementation authority

**Supersedes:** only the obligation-edge, parent-expansion, planned-node, and
Prove2Me-export details of
`2026-09-05-crouzeix-fine-grained-theorem-graph-prd.md`. The route manifests
and published proof receipts remain the proof authority.

## Problem

The Wave-1 obligation graph is useful as an inventory, but it does not yet
justify the stronger reading that its children form a covering decomposition
of their route parent. A `parent-completion` node can bypass open siblings,
obligation edges can name prerequisites outside the bound route node's
dependency surface, and the Prove2Me projection can refer to prerequisites
whose statements are absent from the export. The readback loader also bypasses
the repository's bounded, symlink-safe JSON reader.

The repair must make graph status meaningful without promoting planning
metadata into proof authority. In particular, a passed parent theorem may have
an incomplete fine-grained decomposition: its monolithic Lean proof remains
valid while the expansion remains `expanded-incomplete`.

## Non-goals

- Do not change the three route proof claims or their proof status.
- Do not infer Lean proof-term dependencies from source text.
- Do not add a second handwritten dependency list presented as proof
  authority.
- Do not call Prove2Me or any network service.
- Do not hydrate or update Lean dependencies.
- Do not claim that an inline proof fragment is a separately certified Lean
  declaration.

## Model

### Two kinds of status

`proof_status` continues to describe the bound declaration. A
`parent-completion` node can therefore remain `passed-local` because the route
parent has a receipt even when one of its refinement children is open.

`obligation_status` describes the decomposition:

- `expanded-current`: the parent has exactly one covering completion node,
  that node depends on every other child and no non-child, and every child is
  passed with a current readback.
- `expanded-incomplete`: a structurally covering expansion exists, but one or
  more children is open, blocked, stale, or missing its required readback.

This deliberately separates "the theorem is proved" from "the proof has
been decomposed into independently certified nodes."

### Covering completion invariant

Every expanded parent has exactly one child with `covers_parent: true`. That
child must:

1. have role `parent-completion` or `terminal`;
2. bind the same route node as `parent_node_id`;
3. bind the same Lean name and statement hash as that route node; and
4. depend on all other children of the parent. A completion may additionally
   depend on a passed cross-parent obligation that binds a direct route
   dependency of the parent; it may not bypass a sibling or point at a raw
   route node.

For a covering completion, dependencies are refinement edges. Its monolithic
receipt permits open child dependencies, but those children force
`expanded-incomplete`. Every other passed obligation continues to reject open
or missing dependencies.

### Edge fidelity

For a non-covering proof-bearing obligation bound to route node `R`:

- every route-node dependency must be a direct dependency of `R`; and
- every obligation dependency must either be a sibling refinement or bind a
  direct route dependency of `R`.

This validator catches invented cross-route edges such as making the
compression-moments declaration depend on the power-Cauchy declaration. It
does not require equality with the route dependency list: obligation nodes may
intentionally isolate a narrower direct lemma. Route manifests and receipts
remain authoritative for the complete proof boundary.

### Planned extraction provenance

Every `planned-obligation` must carry:

- `proposed_lean_name`; and
- `extraction_source` with a repository-relative Lean `module_path` and an
  existing enclosing `declaration`.

Validation reads the source through the bounded, symlink-safe reader and
requires the enclosing declaration's terminal name to occur in the file. This
does not certify the planned statement. It records where already-formalized
inline reasoning should be extracted.

The dimension-count node remains planned because its requested package combines
the cubature-specific node bound with the existing `finiteDimensional_countL2`
and `finrank_countL2` declarations; that combined statement is not yet a named,
receipted declaration. Its prose and extraction provenance must say so
explicitly.

## Corrected Wave-1 topology

- `harp-l2-witness-compression-moments` drops the invented dependency on
  `harp-l2-witness-power-cauchy-moments`.
- `harp-l2-witness-package` covers every other child of
  `harp-counting-l2-witness-dimension-bound`.
- `harp-recurrence-operator-lower-bound` covers the recurrence siblings.
- `harp-endpoint-target-norm` covers the endpoint siblings.
- `harp-terminal-outer-limit-transfer` is independent of the inner-limit
  transfer.
- `harp-terminal-main-theorem` covers every other terminal child.
- The three planned extraction nodes point to their enclosing Lean
  declarations.

The covering completion rule is the single structural mechanism used for all
four expanded Harp parents.

## Dependency-closed Prove2Me projection

The dry-run export remains deterministic, stdout-only, and offline. For each
candidate card it computes the transitive closure over both obligation and
route nodes. Dependencies that are not candidate cards appear once in a
top-level `prerequisites` array with:

- theorem ID and node kind (`obligation` or `route`);
- Lean name when one exists;
- statement hash and statement paths;
- natural-language readback when available;
- claim ceiling and local proof status;
- allowed axioms and source locators.

Planned nodes may appear as prerequisites, with a null Lean name and an open
status, but never as candidate cards. Validation of the generated projection
must ensure every dependency ID resolves to either a theorem card or a
prerequisite record. No source text is copied into the export.

Because the exported shape changes, its schema version becomes
`crouzeix-prove2me-export/v2`. The obligation ledger and normalized graph gain
new fields and become `crouzeix-proof-obligation-ledger/v2` and
`crouzeix-proof-obligation-graph/v2`. The route graph remains v1.

## Boundary hardening

`theorem-readbacks.json` must be loaded through
`route_validation._read_json`, preserving the same root confinement, size
limit, regular-file, and symlink rejection used by the other graph inputs.
Boundary errors are translated to `TheoremGraphError` at the theorem-graph
API boundary.

## Verification

Focused tests must demonstrate, failing before implementation and passing
afterward:

- rejection of a symlinked readback ledger;
- rejection of an invented dependency edge;
- rejection of zero or multiple covering completion nodes;
- rejection of a completion node that omits or adds a child;
- acceptance of a passed monolithic completion with open refinement children,
  while reporting `expanded-incomplete`;
- rejection of missing or invalid planned extraction provenance;
- independence of terminal inner and outer limit nodes; and
- dependency closure of every Prove2Me card.

Run while iterating:

```sh
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_theorem_graph
python3 labs/crouzeix_proof_reproduction/proof_evidence.py proof-obligation-graph
python3 labs/crouzeix_proof_reproduction/proof_evidence.py prove2me-export --dry-run
```

Run `mise run verify` once the candidate is frozen. Refresh
`docs/import-receipt.md` only if repository verification reports a payload
digest mismatch, using the digest emitted by `harp repository verify`.

## Acceptance criteria

1. The route graph remains complete with no open, blocked, or missing-readback
   route nodes.
2. Every expanded Harp parent has exactly one structurally covering completion
   node.
3. Parent expansion cannot report `expanded-current` while bypassing a child.
4. Non-covering proof-bearing obligation edges stay within their bound route
   dependency surface.
5. Planned nodes identify an existing enclosing Lean declaration without being
   represented as separately proved.
6. Every Prove2Me dependency resolves inside the v2 export.
7. Readback JSON uses the safe repository boundary.
8. Focused tests and `mise run verify` pass.
9. No Prove2Me or other network action occurs.
