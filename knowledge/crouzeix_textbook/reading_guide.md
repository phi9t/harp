---
id: crouzeix-textbook-reading-guide
title: Reading guide
type: guide
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, pedagogy, proofs, lean]
confidence: high
canonical: reading_guide.md
---

# Reading guide

Back to the [[knowledge/crouzeix_textbook/crouzeix_textbook_index|book index]].

## Normal route

Read each chapter in order. Before a proof, state what the hypotheses buy you.
After the proof, identify the step that would fail if one hypothesis were
removed. Work at least the calculation, one written proof, and the Lean
exercise before advancing.

The recurring chapter rhythm is deliberate:

1. an opening problem creates a need;
2. the conceptual model names the objects;
3. formal development fixes definitions and proves results;
4. examples expose the mechanism;
5. the ML bridge translates without changing the mathematics;
6. Lean makes quantifiers and hypotheses explicit;
7. exercises test recall, calculation, proof, boundaries, and formalization;
8. the synthesis records what later chapters may use.

## Reading the Harp route

[[knowledge/crouzeix_textbook/part_06_constant_two_routes/36_harp_finite_horizon_proof|Chapter 36]]
builds a finite atomic dilation at each requested horizon. Start with its
prerequisite recap, then follow the six numbered arguments in order. Chapters
26 and 28 supply the boundary moments; Chapters 33 and 34 explain the shared
recurrence and dilation tools. The Jin terminal proof is not a prerequisite.

Keep two lists while reading: quantities fixed for the whole argument, and
quantities that may change with the horizon. The target operator, companion
family, and recurrence scalars belong to the first list. Cubature nodes,
dilation spaces, and squared displacements belong to the second. The proof's
uniform estimate is what lets the finite certificates imply a single bound.

## Fast-recall route

Readers with strong prior linear algebra may first read the opening problem,
numbered CFT items, worked examples, Lean declaration types, and synthesis.
Attempt the boundary exercise. If its answer is not immediate, return to the
full formal development. Fast recall is a diagnostic route, not permission to
skip a prerequisite that has become rusty.

## How to read a proof

Separate four layers:

- **Objects:** What types of things occur?
- **Claims:** What is quantified, and in what order?
- **Mechanism:** Which equality or inequality performs the real work?
- **Boundary:** What tempting stronger statement is false?

In Lean, read the declaration before the tactic script. The declaration is
the claim. A short script can prove a strong statement, while a long script can
prove a weak one. The theorem type, not line count, sets the mathematical
content.

## How to use the ML bridges

The bridges assume experience with representations, Jacobians, conditioning,
optimization, and numerical experiments. They provide motivation and transfer
tests. They do not assert that a nonlinear learned representation is a vector
space, that a reparameterization is invertible, or that floating-point code
inherits an exact theorem without an error analysis.

## Proof-assistant boundary

The [[knowledge/crouzeix_textbook/lean_coverage_ledger|coverage ledger]] tells
you exactly which declaration implements each numbered item. Exercise
solutions map to chapter-scoped public declarations listed in the exercise
contract. Compilation checks the formal derivation against Lean's kernel and
the pinned Mathlib version. The
[[knowledge/crouzeix_textbook/status_and_scope|status page]] records the
remaining model-correspondence and review boundaries.
