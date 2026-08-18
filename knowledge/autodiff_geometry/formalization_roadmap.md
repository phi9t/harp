---
id: autodiff-geometry-formalization-roadmap
title: Autodiff geometry formalization roadmap
type: formalization-map
status: active
created: 2026-08-18
updated: 2026-08-18
tags: [autodiff, jax, differential-geometry, lean, roadmap]
confidence: medium
---

# Autodiff geometry formalization roadmap

Back to the [[knowledge/autodiff_geometry/autodiff_geometry_index|packet index]].

This roadmap turns the JAX autodiff cookbook's notation contract into staged
Lean work. A named theorem below is a compiled declaration. A roadmap row with
no Lean declaration is a planning boundary, not a proof claim.

## Compiled declaration inventory

The first Lean phase lives in `formalization/autodiff_geometry`.

| Module | Compiled public declarations |
|---|---|
| `AutodiffGeometry.FiniteCoordinates` | `AutodiffGeometry.grad_arg_eq_component`; `AutodiffGeometry.jvp_eq_matVec`; `AutodiffGeometry.vjp_eq_transpose_matVec`; `AutodiffGeometry.hvp_eq_matVec`; `AutodiffGeometry.dot_jvp_eq_dot_vjp` |

## Phase 1 -- finite-coordinate notation contracts

Status: `compiled`.

| Cookbook surface | Formalized contract | Lean declaration | Boundary |
|---|---|---|---|
| `grad(f, i)` as `partial_i f` | A finite-coordinate gradient component selector returns the `i`th coordinate. | `AutodiffGeometry.grad_arg_eq_component` | The gradient vector is supplied as data; no derivative operator is constructed. |
| `jvp(f, x, v)` as `partial f(x) v` | A JVP is matrix-vector multiplication by a Jacobian matrix. | `AutodiffGeometry.jvp_eq_matVec` | The Jacobian matrix is supplied as data; no JAX primitive rule is modeled. |
| `vjp(f, x)` as a cotangent pullback | A VJP is matrix-vector multiplication by the transposed Jacobian. | `AutodiffGeometry.vjp_eq_transpose_matVec` | Cotangent spaces are finite coordinate vectors; no pytrees or dual-space API are modeled. |
| Hessian-vector products | A Hessian-vector product is matrix-vector multiplication by a square Hessian matrix. | `AutodiffGeometry.hvp_eq_matVec` | The Hessian is supplied as data; no second-derivative construction is included. |
| JVP/VJP duality | The dot product of a pushed tangent with an output cotangent equals the dot product of the input tangent with the pulled cotangent. | `AutodiffGeometry.dot_jvp_eq_dot_vjp` | Finite sums over coordinate vectors, over a commutative semiring. |

## Phase 2 -- differentiability over finite real spaces

Status: `planned`.

Goal: replace supplied Jacobian and Hessian data with mathlib differentiability
objects over finite-dimensional real normed spaces.

Candidate contracts:

- connect a Frechet derivative to the finite-coordinate Jacobian;
- prove `grad` as the Riesz-represented derivative for scalar-output functions
  under a chosen Euclidean inner product;
- construct Hessian-vector products from iterated derivatives under explicit
  smoothness assumptions; and
- state the exact conditions under which Hessian symmetry is available.

Evidence needed:

- mathlib calculus API survey for `HasFDerivAt`, `fderiv`, `ContDiff`,
  gradients, and Hessians;
- a proof decision on whether the finite-coordinate project should reuse the
  existing `mathematical_foundations` Lean project or stay standalone; and
- examples that keep theorem names aligned with cookbook notation without
  expanding into JAX runtime semantics.

## Phase 3 -- JAX API correspondence tests

Status: `planned`.

Goal: build executable examples that compare JAX numeric output against the
finite-coordinate formulas for small functions.

Candidate examples:

- scalar `grad` and `grad(f, i)` on polynomial functions;
- `jvp` and `vjp` for affine and quadratic maps;
- `jacfwd`/`jacrev` shape parity for small dense functions; and
- `hessian-vector product` parity through `jvp(grad(f), ...)`.

Boundary: these tests would validate example correspondence, not prove JAX's
implementation. They must also avoid making JAX a Harp runtime dependency
unless the project owner explicitly approves a dependency plan.

## Phase 4 -- notation lineage and manifold semantics

Status: `evidence-gated`.

Goal: promote the Spivak/SICM/FDG notation lineage from JAX-cited source claim
to direct Harp evidence.

Required before promotion:

- capture or cite source-owned pages/sections for the specific notation in
  Spivak, SICM, and FDG;
- clarify redistribution rights for any textbook bytes;
- record exact locators and claim ceilings; and
- decide whether the formal target is finite-dimensional calculus, smooth
  manifolds, or functional differential geometry.

Boundary: until those captures exist, Harp cites JAX's bibliographic statement
as the source of the lineage claim and does not reproduce book content.

## Phase 5 -- pytrees and product structures

Status: `research`.

Goal: formalize the product-structure intuition behind JAX pytrees, `argnums`,
and container-valued Jacobians.

Candidate contracts:

- finite products as coordinate bundles;
- `argnums` as projection to one product component;
- block Jacobians for product domains and codomains; and
- structured cotangent pullbacks for nested finite products.

Boundary: this is not yet a claim about arbitrary Python containers, registered
custom pytree nodes, or JAX transformation internals.
