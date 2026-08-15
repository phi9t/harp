---
id: mathematical-foundations-index
title: Mathematical foundations for machine learning
type: index
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [mathematics, linear-algebra, probability, machine-learning]
confidence: high
---

# Mathematical foundations for machine learning

This packet is a compact working foundation for readers who program and know
basic calculus. It emphasizes the moves that recur in machine learning:
representing a dataset as a linear map, treating uncertainty as a distribution,
and turning a modelling objective into a computation.

## Source and exercise boundary

The two books named in the [source registry](source_registry.md) were supplied
locally for orientation only. Their files are not part of Harp, are not linked
here, and are not redistributed. This packet contains newly authored
exposition and original exercises; it reproduces no textbook exercise,
solution, or extended passage. The [claim ledger](claim_evidence_ledger.md)
records only high-level source locators.

## Core route

1. [Linear spaces, maps, bases, and matrices](01_linear_spaces_and_maps.md)
2. [Orthogonality, spectra, positive-definite matrices, and decompositions](02_orthogonality_spectra_and_decompositions.md)
3. [Probability, expectation, covariance, and Gaussian models](03_probability_and_gaussian_models.md)
4. [Conditional probability, Bayes, information, and model selection](04_bayesian_inference_and_information.md)
5. [Regression, classification, regularization, and probabilistic models](05_linear_models_and_regularization.md)
6. [Gradients, Hessians, optimization, and iterative methods](06_optimization_and_iterative_methods.md)

The six modules each contain eight short original problems with a worked
solution immediately following the question. Take them in order on a first
pass; use the glossary as a quick lookup rather than a substitute for solving.

## Supporting documents

- [Curriculum map](curriculum_map.md): all chapters of the two supplied books,
  divided into this core and named later tracks.
- [Glossary](glossary.md): compact notation and definitions.
- [Source registry](source_registry.md): identity and copyright boundary.
- [Claim and evidence ledger](claim_evidence_ledger.md): bounded, material
  claims with stable book locators.

## How to study

For each problem, write a one-line prediction before reading the solution.
Then translate the result into code: use vectors for observations, matrices for
collections of features, and a scalar objective for the quantity being
optimized. The point is not memorizing formulas; it is recognizing which
structure makes a formula inevitable.
