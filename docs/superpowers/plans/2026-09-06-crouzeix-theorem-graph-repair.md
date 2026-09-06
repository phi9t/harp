# Crouzeix Theorem-Graph Repair Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use phi9t_stack parallel-agents (recommended) or phi9t_stack plan to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the fine-grained Harp obligation DAG a validated covering
refinement and make its offline Prove2Me projection dependency-closed.

**Architecture:** Preserve the route graph and receipts as proof authority. Add
explicit covering-completion and extraction-provenance fields to obligation
nodes, validate refinement structure separately from monolithic proof status,
and project the transitive dependency closure into the offline export.

**Tech Stack:** Python 3 standard library, JSON ledgers, `unittest`, existing
Harp route validation helpers.

---

### Task 1: Harden readback loading

**Files:**
- Modify: `labs/crouzeix_proof_reproduction/tests/test_theorem_graph.py`
- Modify: `labs/crouzeix_proof_reproduction/theorem_graph.py`

- [ ] Add a test that copies the minimum graph inputs to a temporary root,
  replaces `theorem-readbacks.json` with a symlink, and expects a
  `TheoremGraphError`.
- [ ] Run that test and observe the old raw reader accept the symlink or fail
  with the wrong boundary behavior.
- [ ] Route `_load_readbacks` through `route_validation._read_json` and translate
  `RouteValidationError` to `TheoremGraphError`.
- [ ] Run the focused theorem-graph suite.

### Task 2: Add covering refinements and planned provenance

**Files:**
- Modify: `labs/crouzeix_proof_reproduction/tests/test_theorem_graph.py`
- Modify: `labs/crouzeix_proof_reproduction/theorem_graph.py`
- Modify: `labs/crouzeix_proof_reproduction/formal_targets/proof-obligations.json`

- [ ] Add failing tests for the v2 fields, unique covering completion, exact
  sibling coverage, completion binding, edge fidelity, and extraction-source
  validation.
- [ ] Extend the parsed and serialized obligation node with `covers_parent`,
  `proposed_lean_name`, and `extraction_source`.
- [ ] Implement the covering-completion, edge-fidelity, and planned-provenance
  validators. Exempt only a covering completion from rejection solely because
  one of its refinement children is open.
- [ ] Correct all four Harp parent expansions and the known spurious/independent
  edges in the ledger.
- [ ] Assert exact parent statuses and frontier contents in the focused tests.
- [ ] Run the focused theorem-graph suite and both graph CLI commands.

### Task 3: Export a closed collaboration packet

**Files:**
- Modify: `labs/crouzeix_proof_reproduction/tests/test_theorem_graph.py`
- Modify: `labs/crouzeix_proof_reproduction/theorem_graph.py`

- [ ] Add a failing test that gathers every card dependency and requires its ID
  in either `theorem_cards` or `prerequisites`.
- [ ] Add deterministic transitive-closure traversal across obligation and route
  nodes.
- [ ] Emit normalized prerequisite records and reject unresolved dependencies.
- [ ] Assert `dry_run: true` and `network_access: false` remain unchanged.
- [ ] Run the focused suite twice and compare export bytes.

### Task 4: Verify, review, and land locally

**Files:**
- Modify only if required: `docs/import-receipt.md`
- Create: `docs/superpowers/reviews/2026-09-06-crouzeix-theorem-graph-repair-standards.md`
- Create: `docs/superpowers/reviews/2026-09-06-crouzeix-theorem-graph-repair-spec.md`

- [ ] Run `python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_theorem_graph`.
- [ ] Run the graph and export CLI smoke tests and parse their stdout as JSON.
- [ ] Run `mise run verify`.
- [ ] Run independent Standards and Spec reviews against commit `f7bf0b49`;
  repair every high-confidence finding and rerun affected tests.
- [ ] Stage explicit paths only and commit the coherent repair. Do not push.
