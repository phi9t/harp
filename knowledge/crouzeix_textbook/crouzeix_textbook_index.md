---
id: crouzeix-textbook-index
title: Crouzeix foundations textbook
type: index
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, mathematics, linear-algebra, operator-theory, lean]
confidence: high
canonical: crouzeix_textbook_index.md
---

# Crouzeix foundations textbook

This is a proof-oriented route from structural linear algebra to the two
constant-two Crouzeix proof programs. It assumes undergraduate mathematical
maturity and the habits of a machine-learning researcher, but it does not
assume recent coursework in linear algebra, complex analysis, or operator
theory. Definitions are rebuilt before they carry weight.

The book is text-only. Every chapter pairs informal mathematics with a
compiled Lean 4 surface and original exercises. Lean checks the formal
statements; it does not by itself certify that the prose is readable, that a
formal statement perfectly models its informal counterpart, or that a recent
candidate proof has completed independent review.

## Start here

- [[knowledge/crouzeix_textbook/reading_guide|Reading guide]]
- [[knowledge/crouzeix_textbook/notation_and_glossary|Notation and glossary]]
- [[knowledge/crouzeix_textbook/status_and_scope|Status and scope]]
- [[knowledge/crouzeix_textbook/lean_coverage_ledger|Lean coverage ledger]]
- [[knowledge/crouzeix_textbook/theorem_dependency_map|Theorem dependency map]]
- [[knowledge/crouzeix_textbook/exercise_index|Exercise index]]
- [[knowledge/crouzeix_textbook/source_registry|Source registry]]
- [[knowledge/crouzeix_textbook/claim_evidence_ledger|Claim-evidence ledger]]

## Part I -- Linear structure

The active route begins with

1. [[knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations|Objects and representations]]
2. [[knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces|Vector spaces and subspaces]]
3. [[knowledge/crouzeix_textbook/part_01_linear_structure/03_linear_maps_and_exact_structure|Linear maps and exact structure]]
4. [[knowledge/crouzeix_textbook/part_01_linear_structure/04_coordinates_and_duality|Coordinates and duality]]
5. [[knowledge/crouzeix_textbook/part_01_linear_structure/05_determinants_trace_and_exterior_algebra|Determinants, trace, and exterior algebra]]
6. [[knowledge/crouzeix_textbook/part_01_linear_structure/06_eigenvalues_and_polynomial_algebra|Eigenvalues and polynomial algebra]]

Part I is active and closes the structural path from basis-free maps to
simple-spectrum polynomial action.

## Part II -- Geometry and calculus

7. [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/07_inner_product_spaces|Inner-product spaces]]
8. [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/08_positive_operators_and_gram_geometry|Positive operators and Gram geometry]]
9. [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/09_operator_norms_and_singular_values|Operator norms and singular values]]
10. [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/10_multilinear_maps_and_tensors|Multilinear maps and tensors]]
11. [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/11_differentiation_as_linear_approximation|Differentiation as linear approximation]]
12. [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/12_differential_forms_and_stokes|Differential forms and Stokes]]

Part II is active. Its formal boundary is finite-coordinate and planar where
the broader manifold API is not needed by the later proof.

## Part III -- Analysis and complex functions

13. [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/13_metric_and_normed_spaces|Metric and normed spaces]]
14. [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/14_sequences_and_series_of_operators|Sequences and series of operators]]
15. [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/15_complex_differentiability|Complex differentiability]]
16. [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/16_consequences_of_cauchy_theory|Consequences of Cauchy theory]]
17. [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/17_functions_of_matrices_and_operators|Functions of matrices and operators]]
18. [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/18_positive_real_analytic_functions|Positive-real analytic functions]]

Part III is active. It separates algebraic identities from convergence,
builds the contour functional calculus, and ends at the matrix Herglotz
positivity interface used by the completion proof.

## Part IV -- Finite-dimensional operator theory

19. [[knowledge/crouzeix_textbook/part_04_operator_theory/19_normality_and_nonnormality|Normality and nonnormality]]
20. [[knowledge/crouzeix_textbook/part_04_operator_theory/20_numerical_range|The numerical range]]
21. [[knowledge/crouzeix_textbook/part_04_operator_theory/21_spectral_sets|Spectral sets]]
22. [[knowledge/crouzeix_textbook/part_04_operator_theory/22_positive_and_completely_positive_maps|Positive and completely positive maps]]
23. [[knowledge/crouzeix_textbook/part_04_operator_theory/23_compression_and_dilation|Compression and dilation]]
24. [[knowledge/crouzeix_textbook/part_04_operator_theory/24_gramians_and_ordered_matrix_inequalities|Gramians and ordered matrix inequalities]]

Part IV is active and ends with the ordered Gramian difference used in the
positive-real completion endpoint.

## Part V -- Crouzeix machinery

25. [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/25_convex_boundaries_and_cauchy_layers|Convex boundaries and Cauchy layers]]
26. [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/26_double_layer_map|The double-layer map]]
27. [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/27_one_plus_sqrt_two_barrier|The one-plus-square-root-two barrier]]
28. [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/28_complete_power_family|The complete power family]]

Part V is active. It isolates the common fixed-domain machinery before the
two sharp routes diverge.

## Part VI -- Constant-two routes

29. [[knowledge/crouzeix_textbook/part_06_constant_two_routes/29_crouzeix_problem_and_sharpness|The Crouzeix problem and sharpness]]
30. [[knowledge/crouzeix_textbook/part_06_constant_two_routes/30_jin_positive_real_completion|Jin's positive-real completion]]
31. [[knowledge/crouzeix_textbook/part_06_constant_two_routes/31_jin_correction_cancellation|Jin's correction cancellation]]
32. [[knowledge/crouzeix_textbook/part_06_constant_two_routes/32_jin_constant_two_endpoint|Jin's constant-two endpoint]]
33. [[knowledge/crouzeix_textbook/part_06_constant_two_routes/33_lorist_schwenninger_perturbation_lemma|The Lorist--Schwenninger perturbation lemma]]
34. [[knowledge/crouzeix_textbook/part_06_constant_two_routes/34_lorist_schwenninger_realization|The Lorist--Schwenninger realization]]
35. [[knowledge/crouzeix_textbook/part_06_constant_two_routes/35_comparison_verification_and_boundaries|Comparison, verification, and boundaries]]

Part VI is active. Both terminal finite-matrix providers and their neutral
rational and Hilbert-space consequences are represented by compiled Lean
declarations.

## Completion contract

All 35 chapters and 210 exercises are active. The structured ledgers map 210
numbered mathematical items to the public Lean inventory. “Complete” refers
to this repository contract and its verification gates; it is not a claim of
independent peer review for the recent source proofs.
