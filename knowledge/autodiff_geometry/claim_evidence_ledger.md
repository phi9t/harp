---
id: autodiff-geometry-claim-evidence-ledger
title: Autodiff geometry claim and evidence ledger
type: claim-ledger
status: active
created: 2026-08-18
updated: 2026-08-18
tags: [autodiff, jax, claims, evidence, lean]
confidence: medium
---

# Autodiff geometry claim and evidence ledger

Back to the [[knowledge/autodiff_geometry/autodiff_geometry_index|packet index]].

Material claims use one of these classes:

- `EVIDENCE` for source-backed or compiled facts;
- `SOURCE CLAIM` for upstream wording not locally reproduced;
- `INFERENCE` for Harp-authored synthesis over evidence; and
- `MISSING` for a claim that needs stronger evidence before promotion.

## ADG-001: JAX `grad` names the mathematical gradient

- Class: `EVIDENCE`
- Statement: The JAX autodiff cookbook says `grad(f)` is a Python function for
  evaluating the mathematical gradient `nabla f`, and `grad(f)(x)` represents
  `nabla f(x)`.
- Source: [JAX-AUTODIFF-COOKBOOK](source_registry.md#source-classes-and-boundary)
- Locator: Pinned notebook lines containing `grad(f)` / `nabla f`, observed in
  `docs/notebooks/autodiff_cookbook.ipynb` at commit
  `0eb0f676ba22b15b9dfe29518ab0d40277b7138d`.
- Scope: Public documentation notation for scalar-output differentiation.
- Confidence: `high`

## ADG-002: JAX `grad(f, i)` corresponds to partial derivative notation

- Class: `EVIDENCE`
- Statement: The cookbook explicitly says the `grad` API has direct
  correspondence to notation in Spivak, SICM, and FDG, and that `grad(f, i)`
  evaluates `partial_i f`.
- Source: [JAX-AUTODIFF-COOKBOOK](source_registry.md#source-classes-and-boundary)
- Locator: Pinned notebook lines containing `direct correspondence`,
  `Spivak`, and `grad(f, i)` / `partial_i f`.
- Scope: Documentation notation claim, not a proof that the books use the same
  formal definitions in every context.
- Confidence: `high`

## ADG-003: JAX `jvp` and `vjp` expose pushforward and pullback views

- Class: `EVIDENCE`
- Statement: The cookbook describes `jvp` as evaluating
  `(x, v) -> (f(x), partial f(x) v)` and `vjp` as producing a linear map for
  vector-Jacobian products, written as `v^T partial f(x)`.
- Source: [JAX-AUTODIFF-COOKBOOK](source_registry.md#source-classes-and-boundary)
- Locator: Pinned notebook lines for `jvp :: (a -> b) -> a -> T a -> (b, T b)`
  and `vjp :: (a -> b) -> a -> (b, CT b -> CT a)`.
- Scope: API-level mathematical description over tangent and cotangent values.
- Confidence: `high`

## ADG-004: JAX API source exposes the named transformations

- Class: `EVIDENCE`
- Statement: In the pinned JAX source, `jax/_src/api.py` defines public
  transformation entry points named `grad`, `value_and_grad`, `jacfwd`,
  `jacrev`, `hessian`, `jvp`, and `vjp`.
- Source: [JAX-API-SOURCE](source_registry.md#source-classes-and-boundary)
- Locator: `jax/_src/api.py` at commit
  `0eb0f676ba22b15b9dfe29518ab0d40277b7138d`; definition lines observed for
  the named functions.
- Scope: Source identity and API-surface location, not correctness of the
  implementation.
- Confidence: `high`

## ADG-005: JAX efficiency claims are prose-only for now

- Class: `SOURCE CLAIM`
- Statement: The cookbook says `jacfwd` is more efficient for tall Jacobians,
  `jacrev` for wide Jacobians, and forward-over-reverse is typically efficient
  for dense Hessians.
- Source: [JAX-AUTODIFF-COOKBOOK](source_registry.md#source-classes-and-boundary)
- Locator: Pinned notebook lines discussing `jacfwd`, `jacrev`, tall/wide
  Jacobians, and `jacfwd(jacrev(f))`.
- Scope: Upstream documentation guidance. Harp has not benchmarked or
  reproduced these complexity claims.
- Confidence: `medium`

## ADG-006: Harp compiles finite-coordinate autodiff operator contracts

- Class: `EVIDENCE`
- Statement: The Lean project `formalization/autodiff_geometry` compiles
  theorem declarations for gradient component selection, JVP as matrix-vector
  multiplication, VJP as transposed matrix-vector multiplication,
  Hessian-vector multiplication, and finite-coordinate JVP/VJP duality.
- Source: `formalization/autodiff_geometry/AutodiffGeometry/FiniteCoordinates.lean`
- Locator: Lean declarations `grad_arg_eq_component`, `jvp_eq_matVec`,
  `vjp_eq_transpose_matVec`, `hvp_eq_matVec`, and `dot_jvp_eq_dot_vjp`.
- Scope: Finite coordinate vectors over a commutative semiring. No analytic
  differentiability, floating-point, or JAX tracing semantics.
- Confidence: `high`

## ADG-007: Spivak, SICM, and FDG are notation lineage, not local proof evidence

- Class: `INFERENCE`
- Statement: In this packet, Spivak, SICM, and FDG serve as notation-lineage
  anchors because JAX names them as such; they do not yet provide local theorem
  evidence beyond bibliographic/source pointers.
- Sources:
  - [JAX-AUTODIFF-COOKBOOK](source_registry.md#source-classes-and-boundary)
  - [SPIVAK-CALCULUS-ON-MANIFOLDS-REPO](source_registry.md#source-classes-and-boundary)
  - [SICM-COURSE-PAGE](source_registry.md#source-classes-and-boundary)
- Scope: Evidence-boundary rule for Harp. Later direct captures may promote
  specific textbook claims.
- Confidence: `medium`

## ADG-008: Manifold-level and implementation correspondence are missing

- Class: `MISSING`
- Statement: Harp does not yet have compiled manifold-level differential
  geometry theorems, pytrees-as-product-manifolds contracts, JAX primitive rule
  correspondence, complex holomorphic differentiation contracts, or executable
  JAX/Lean example parity.
- Source: [Formalization roadmap](formalization_roadmap.md)
- Scope: Explicit non-claim boundary for future work.
- Confidence: `high`
