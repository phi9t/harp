---
id: autodiff-geometry-index
title: Autodiff geometry formalization
type: formalization-index
status: active
created: 2026-08-18
updated: 2026-08-18
tags: [autodiff, jax, differential-geometry, lean, formalization]
confidence: medium
---

# Autodiff geometry formalization

Mode: `FORMALIZATION INDEX`.

This packet starts a source-backed bridge from JAX autodiff notation to
finite-coordinate differential-operator contracts. The first compiled Lean
phase is deliberately narrow: it proves algebraic correspondences for
component gradients, JVPs, VJPs, Hessian-vector products, and JVP/VJP duality.

## Reader routes

- [[knowledge/autodiff_geometry/source_registry|Source registry]]
- [[knowledge/autodiff_geometry/claim_evidence_ledger|Claim evidence ledger]]
- [[knowledge/autodiff_geometry/formalization_roadmap|Formalization roadmap]]

## Core framing

**EVIDENCE -- [ADG-001], [ADG-002], [ADG-003].** JAX's autodiff cookbook states
the notation contract we want to preserve: `grad(f)` evaluates the mathematical
gradient, `grad(f, i)` evaluates a partial derivative, `jvp` evaluates
`(x, v) -> (f(x), partial f(x) v)`, and `vjp` evaluates the pullback
`(x, v) -> (f(x), v^T partial f(x))`.

**EVIDENCE -- [ADG-006].** Harp now compiles a finite-coordinate Lean phase for
the algebra under that notation. It does not claim to verify JAX tracing,
floating-point numerics, pytrees, complex holomorphic differentiation, or any
textbook theorem beyond the named finite-coordinate declarations.

## Claim ceiling

This packet supports:

- source-backed orientation to the JAX autodiff cookbook's notation and API
  surfaces;
- a finite-coordinate Lean formalization of the algebraic operators named by
  the cookbook; and
- a roadmap for later phases that can add differentiability and implementation
  correspondence under explicit evidence boundaries.

This packet does not support:

- a claim that Harp verified the JAX implementation;
- reproduction of cookbook numerical examples;
- copied textbook exposition or exercise material;
- claims about MIT Press book pages that were not accessible to plain HTTP
  capture in this environment; or
- manifold-level or infinite-dimensional differential geometry theorems.
