---
id: math-foundations-formalization-map
title: Mathematical foundations formalization map
type: formalization-map
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [mathematics, formalization, lean, machine-learning]
confidence: high
---

# Mathematical foundations formalization map

Back to the [packet index](mathematical_foundations_index.md). This is the
reader-facing boundary for the accompanying Lean project: a named theorem is a
compiled statement over its stated domain, while a prose-only row makes no
Lean-proof claim. The finite-coordinate formalization uses expressions such as
$x \in \mathbb R^n$; it is not a proof of an empirical modelling conclusion.

## Status key

- **Direct theorem** — the original problem statement is a compiled named
  theorem.
- **Corollary/application** — the worked solution uses a compiled theorem, but
  still needs the displayed instance or calculation.
- **Prose-only** — the row states an exact boundary and has no Lean identifier.

## Compiled declaration inventory

The following are the public compiled theorems in the six supported namespaces.
They are listed here so the map does not imply that unlisted Lean results exist.

| Module | Compiled public theorems |
| --- | --- |
| [Module 1: linear spaces and maps](01_linear_spaces_and_maps.md) | `MathematicalFoundations.Linear.linear_map_zero`; `MathematicalFoundations.Linear.linear_map_add`; `MathematicalFoundations.Linear.linear_map_smul`; `MathematicalFoundations.Linear.linear_kernel_zero`; `MathematicalFoundations.Linear.linear_kernel_add`; `MathematicalFoundations.Linear.linear_kernel_smul`; `MathematicalFoundations.Linear.linear_finite_coordinate_reconstruction` |
| [Module 2: orthogonality](02_orthogonality_spectra_and_decompositions.md) | `MathematicalFoundations.Orthogonality.line_projection_residual_decomposition`; `MathematicalFoundations.Orthogonality.line_projection_residual_inner_eq_zero`; `MathematicalFoundations.Orthogonality.symmetric_positive_definite_quadratic_positive` |
| [Module 3: probability](03_probability_and_gaussian_models.md) | `MathematicalFoundations.Probability.expectation_add`; `MathematicalFoundations.Probability.expectation_smul`; `MathematicalFoundations.Probability.expectation_sub`; `MathematicalFoundations.Probability.expectation_const`; `MathematicalFoundations.Probability.covariance_expand` |
| [Module 4: Bayes and information](04_bayesian_inference_and_information.md) | `MathematicalFoundations.BayesInformation.eventProbability_and_comm`; `MathematicalFoundations.BayesInformation.conditional_mul_evidence`; `MathematicalFoundations.BayesInformation.bayes_rule` |
| [Module 5: linear models](05_linear_models_and_regularization.md) | `MathematicalFoundations.LinearModels.normal_equation_residual_identity`; `MathematicalFoundations.LinearModels.ridge_objective_zero_coefficients` |
| [Module 6: optimization](06_optimization_and_iterative_methods.md) | `MathematicalFoundations.Optimization.scalar_gradient_descent_error_recurrence`; `MathematicalFoundations.Optimization.scalar_stationary_iteration_succ`; `MathematicalFoundations.Optimization.scalar_stationary_iteration_tendsto_fixed_point` |

## Module 1 — linear spaces and maps

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-01-01 | Prose-only | — | Exact limitation: the numerical coordinate solve for these three vectors has no compiled declaration. |
| MF-01-02 | Corollary/application | Lean: `MathematicalFoundations.Linear.linear_map_add`; `MathematicalFoundations.Linear.linear_map_smul` | The column-combination explanation is an application; the displayed matrix-vector arithmetic is not separately formalized. |
| MF-01-03 | Corollary/application | Lean: `MathematicalFoundations.Linear.linear_kernel_zero`; `MathematicalFoundations.Linear.linear_kernel_add`; `MathematicalFoundations.Linear.linear_kernel_smul` | Kernel closure supports the subspace portion; the particular two-vector basis is not separately formalized. |
| MF-01-04 | Prose-only | — | Exact limitation: the matrix-column dependence interpretation is not represented by a compiled theorem. |
| MF-01-05 | Prose-only | — | Exact limitation: this concrete inverse calculation is not represented by a compiled theorem. |
| MF-01-06 | Prose-only | — | Exact limitation: rank and row-space dimension are outside the companion's theorem boundary. |
| MF-01-07 | Corollary/application | Lean: `MathematicalFoundations.Linear.linear_finite_coordinate_reconstruction` | Coordinatewise equality supports reconstruction after the displayed numerical solve. |
| MF-01-08 | Prose-only | — | Exact limitation: no compiled idempotent-matrix theorem is included. |

## Module 2 — orthogonality, spectra, and decompositions

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-02-01 | Corollary/application | Lean: `MathematicalFoundations.Orthogonality.line_projection_residual_decomposition`; `MathematicalFoundations.Orthogonality.line_projection_residual_inner_eq_zero` | The residual decomposition and orthogonality are compiled for finite real vectors; the displayed numeric projection still needs arithmetic. |
| MF-02-02 | Prose-only | — | Exact limitation: no compiled eigenvalue calculation or spectral theorem is included. |
| MF-02-03 | Prose-only | — | Exact limitation: no compiled orthogonal-matrix verification for this concrete matrix is included. |
| MF-02-04 | Prose-only | — | Exact limitation: the companion does not prove positive definiteness for this concrete matrix from its entries. |
| MF-02-05 | Prose-only | — | Exact limitation: singular-value maximization is outside the companion's theorem boundary. |
| MF-02-06 | Prose-only | — | Exact limitation: covariance-matrix validity and invertibility for this matrix are not formalized. |
| MF-02-07 | Prose-only | — | Exact limitation: rank-one approximation and Frobenius-error claims are not formalized. |
| MF-02-08 | Prose-only | — | Exact limitation: Cholesky factorization and Gaussian sampling are not formalized. |

## Module 3 — probability and Gaussian models

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-03-01 | Prose-only | — | Exact limitation: the fair-die evaluation is not encoded as a particular finite probability mass function. |
| MF-03-02 | Prose-only | — | Exact limitation: the Bernoulli variance formula is not a compiled theorem. |
| MF-03-03 | Corollary/application | Lean: `MathematicalFoundations.Probability.expectation_add`; `MathematicalFoundations.Probability.expectation_smul`; `MathematicalFoundations.Probability.expectation_sub`; `MathematicalFoundations.Probability.expectation_const` | The finite-PMF expectation transformation is supported; the variance-scaling portion is not a compiled theorem. |
| MF-03-04 | Corollary/application | Lean: `MathematicalFoundations.Probability.covariance_expand` | The covariance expansion supports the two-point calculation after choosing its finite mass function. |
| MF-03-05 | Prose-only | — | Exact limitation: independence and the variance of an average are not formalized. |
| MF-03-06 | Prose-only | — | Exact limitation: Gaussian densities and their ratio are outside the finite-PMF boundary. |
| MF-03-07 | Prose-only | — | Exact limitation: affine transformations of Gaussian distributions are not formalized. |
| MF-03-08 | Prose-only | — | Exact limitation: covariance conditioning and Gaussian-density numerics are not formalized. |

## Module 4 — Bayesian inference and information

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-04-01 | Corollary/application | Lean: `MathematicalFoundations.BayesInformation.eventProbability_and_comm`; `MathematicalFoundations.BayesInformation.conditional_mul_evidence`; `MathematicalFoundations.BayesInformation.bayes_rule` | Bayes' identity is compiled for finite probability mass functions with nonzero prior and nonzero evidence; the displayed decimal calculation remains an application. |
| MF-04-02 | Prose-only | — | Exact limitation: Beta-Bernoulli conjugacy is not formalized. |
| MF-04-03 | Prose-only | — | Exact limitation: MAP and maximum-likelihood optimization are not formalized. |
| MF-04-04 | Prose-only | — | Exact limitation: entropy in bits is not formalized. |
| MF-04-05 | Prose-only | — | Exact limitation: mutual information and its independence identity are not formalized. |
| MF-04-06 | Prose-only | — | Exact limitation: this model-selection judgment is not a compiled mathematical claim. |
| MF-04-07 | Prose-only | — | Exact limitation: posterior predictive integration is not formalized. |
| MF-04-08 | Prose-only | — | Exact limitation: independent-observation likelihood factorization is not formalized. |

## Module 5 — linear models and regularization

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-05-01 | Corollary/application | Lean: `MathematicalFoundations.LinearModels.normal_equation_residual_identity` | The zero-normal-residual algebra is compiled; the least-squares minimizer calculation is still an application. |
| MF-05-02 | Prose-only | — | Exact limitation: full-rank invertibility and the minimizer characterization are not compiled. |
| MF-05-03 | Prose-only | — | Exact limitation: logistic probabilities and their bounds are not formalized. |
| MF-05-04 | Prose-only | — | Exact limitation: binary cross-entropy evaluation is not formalized. |
| MF-05-05 | Prose-only | — | Exact limitation: the comparative sparsity behavior of $\ell_1$ and $\ell_2$ penalties is not formalized. |
| MF-05-06 | Prose-only | — | Exact limitation: the intercept-column modelling interpretation is not a compiled theorem. |
| MF-05-07 | Prose-only | — | Exact limitation: the Gaussian-noise predictive distribution and likelihood equivalence are not formalized. |
| MF-05-08 | Prose-only | — | Exact limitation: validation-curve interpretation is empirical modelling guidance, not a compiled theorem. |

## Module 6 — optimization and iterative methods

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-06-01 | Prose-only | — | Exact limitation: the displayed two-variable gradient calculation is not formalized. |
| MF-06-02 | Corollary/application | Lean: `MathematicalFoundations.Optimization.scalar_gradient_descent_error_recurrence` | The scalar quadratic error recurrence supports the one-step calculation after substituting its parameters. |
| MF-06-03 | Prose-only | — | Exact limitation: the concrete Hessian and positive-definiteness calculation are not formalized. |
| MF-06-04 | Prose-only | — | Exact limitation: the scalar stationary-iteration results prove convergence under an explicit contraction, not divergence from an excessively large gradient-descent learning rate. |
| MF-06-05 | Prose-only | — | Exact limitation: the vector quadratic gradient and linear-system equivalence are not formalized. |
| MF-06-06 | Prose-only | — | Exact limitation: residual monitoring for a matrix iteration is not formalized. |
| MF-06-07 | Prose-only | — | Exact limitation: conjugate-gradient correctness for positive-definite systems is not formalized. |
| MF-06-08 | Prose-only | — | Exact limitation: the minibatch tradeoff is an empirical computational judgment, not a compiled theorem. |
