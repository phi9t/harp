---
id: mathematical-foundations-claim-evidence-ledger
title: Mathematical foundations claim and evidence ledger
type: claim-ledger
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [claims, evidence, mathematics, machine-learning]
confidence: high
---

# Claim and evidence ledger

Back to the [packet index](mathematical_foundations_index.md). These are
material orientation claims, not a replacement for proof. “Paraphrase” means
the statement is newly written; no source text is reproduced.

## MF-CL-001: Linear maps make coordinate computations portable

- Class: `EVIDENCE`
- Statement: A linear transformation can be represented by a matrix after
  bases are chosen, and composition corresponds to matrix multiplication.
- Mode: `paraphrase`
- Source: [LAX-2007](source_registry.md#source-classes-and-boundary)
- Locator: Chapter III, section “Linear Mappings”; printed pp. 19–31;
  PDF pp. 33–45.
- Scope: Finite-dimensional linear algebra.
- Confidence: `high`

## MF-CL-002: Orthogonal decomposition turns least squares into projection

- Class: `EVIDENCE`
- Statement: In an inner-product space, the best approximation from a subspace
  is characterized by a residual orthogonal to that subspace.
- Mode: `paraphrase`
- Source: [LAX-2007](source_registry.md#source-classes-and-boundary)
- Locator: Chapter VII, section “Euclidean Structure”; printed pp. 77–100;
  PDF pp. 91–114.
- Scope: Euclidean least squares and projections.
- Confidence: `high`

## MF-CL-003: Gaussian conditioning is a core modelling operation

- Class: `EVIDENCE`
- Statement: Joint Gaussian assumptions permit analytic conditional means and
  covariances, which makes them a reusable baseline for regression and latent
  variable models.
- Mode: `paraphrase`
- Source: [BISHOP-2006](source_registry.md#source-classes-and-boundary)
- Locator: Chapter 2, §2.3 “The Gaussian Distribution”; printed pp. 44–57;
  PDF pp. 63–76.
- Scope: Multivariate Gaussian models.
- Confidence: `high`

## MF-CL-004: Regularization changes the selected predictor

- Class: `EVIDENCE`
- Statement: Adding a parameter penalty to a data-fit objective expresses a
  preference among fits and changes the optimization problem.
- Mode: `paraphrase`
- Source: [BISHOP-2006](source_registry.md#source-classes-and-boundary)
- Locator: Chapter 3, §3.1 “Linear Basis Function Models”; printed pp. 140–153;
  PDF pp. 159–172.
- Scope: Penalized linear models.
- Confidence: `high`

## MF-CL-005: Curvature guides local optimization

- Class: `EVIDENCE`
- Statement: A local quadratic approximation uses a gradient and Hessian, and
  iterative linear solvers are relevant when forming a full inverse is wasteful.
- Mode: `paraphrase`
- Source: [LAX-2007](source_registry.md#source-classes-and-boundary)
- Locators:
  - Chapter IX, section “Calculus of Vector- and Matrix-Valued Functions”;
    printed pp. 121–142; PDF pp. 135–156.
  - Chapter XVII, section “How to Solve Systems of Linear Equations”;
    printed pp. 246–261; PDF pp. 260–275.
- Scope: Smooth unconstrained optimization and linear systems.
- Confidence: `high`

## Copyright boundary

The locators above identify topics without exposing supplied scans. All
exposition and all 48 exercises with their solutions in this packet are
original Harp-authored learning material, not copied or adapted textbook
exercise content.
