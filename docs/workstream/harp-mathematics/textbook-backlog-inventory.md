# Textbook completion backlog inventory

Package 0 of [the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md).
Recomputed from `content/crouzeix_textbook/coverage.json` and
`content/crouzeix_textbook/exercises.json`, most recently on 2026-09-09 after
Chapters 7 and 8 landed. Every number below is derived from those two contracts,
not copied from a status page.

## Reconciliation against the plan's baseline

The plan's accepted snapshot recorded 96 exact correspondences, 89
reconstructible proofs, 96 solved exercises, and 216 indexed cards. That
snapshot was reproduced exactly from the contracts before this package began,
so no drift existed between the status surface and the structured records.

After Chapters 5 through 8:

| Metric | Plan baseline | Now | Remaining |
| --- | ---: | ---: | ---: |
| Indexed cards | 216 | 216 | identities preserved |
| Exact correspondence | 96 | 120 | 96 |
| Reconstructible proofs | 89 | 113 | 101 summaries |
| Proof not applicable | 2 | 2 | definitions, no obligation |
| Distinct solved exercises | 96 | 120 | 96 |
| Compiler receipt declarations | 419 | 447 | — |

The 447 receipt rows are 216 public card declarations, 120 distinct exercise
solutions, and 111 underlying providers named by `reexported-proof` rows.

## Two axes that are not the same

An `exact` correspondence row asserts that the prose states the Lean type. A
`reconstructible` proof row asserts that the prose derivation can be followed
without consulting the Lean source. A card can have one without the other, and
four cards deliberately do.

Six cards carry `formal_mode: definition`; only two exposition rows use
`not-applicable`. The other four definition-mode cards are `reconstructible`
because their prose develops a result about the defined object. That asymmetry
is intended and is not a missing obligation.

## Previews are separated from unfinished work

Four cards remain `summary` by editorial decision rather than by omission:
`CFT-01-005` (characteristic polynomial), and `CFT-04-004`, `CFT-04-005`,
`CFT-04-006` (the three scalar invariants). Their full derivations are now
owned by Chapters 5 and 6, and each preview links forward to the owning card.
Importing those proofs backwards would make Chapter 4 depend on Chapter 5,
so the previews stay labeled forward references and stay outside the
completed-proof count. `chapters_01_through_06_publish_exact_cards_with_explicit_proof_boundaries`
pins that decision in the test suite.

The remaining 97 summary rows in Chapters 9–24 are unfinished work, not
previews.

## Chapters 9–24: the exact remaining backlog

| Ch | Kata | cards `checkpoint` | cards `unmapped` | summary proofs | unsolved exercises | declared providers already named |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 9 | `wstr` | 1 | 5 | 6 | 6 | 6 |
| 10 | `0wc6` | 2 | 4 | 6 | 6 | 6 |
| 11 | `raf4` | 5 | 1 | 6 | 6 | 6 |
| 12 | `r60f` | 1 | 5 | 6 | 6 | 6 |
| 13 | `fx6b` | 1 | 5 | 6 | 6 | 6 |
| 14 | `2700` | 2 | 4 | 6 | 6 | 6 |
| 15 | `hg37` | 4 | 2 | 6 | 6 | 6 |
| 16 | `92zd` | 1 | 5 | 6 | 6 | 6 |
| 17 | `q81q` | 4 | 2 | 6 | 6 | 6 |
| 18 | `dvm6` | 5 | 1 | 6 | 6 | 6 |
| 19 | `1r2p` | 1 | 5 | 6 | 6 | 6 |
| 20 | `efcd` | 0 | 6 | 6 | 6 | 6 |
| 21 | `gkgr` | 3 | 3 | 6 | 6 | 6 |
| 22 | `17d3` | 2 | 4 | 6 | 6 | 6 |
| 23 | `0f59` | 1 | 5 | 6 | 6 | 6 |
| 24 | `nz6f` | 2 | 4 | 6 | 6 | 6 |
| **9–24** | | **35** | **61** | **97** | **96** | **96** |

### Per-identity backlog, Chapters 9–24

| ID | correspondence | proof | formal mode | named declaration | exercise |
| --- | --- | --- | --- | --- | --- |
| CFT-09-001 | unmapped | summary | reexported-proof | `Part02.induced_matrix_norm_identity` | CFT-09-E01 unsolved |
| CFT-09-002 | unmapped | summary | reexported-proof | `Part02.polar_factor_is_unitary` | CFT-09-E02 unsolved |
| CFT-09-003 | unmapped | summary | reexported-proof | `Part02.polar_similarity_norm_transfer` | CFT-09-E03 unsolved |
| CFT-09-004 | unmapped | summary | reexported-proof | `Part02.polar_operator_norm_transfer` | CFT-09-E04 unsolved |
| CFT-09-005 | unmapped | summary | reexported-proof | `Part02.quadratic_bound_implies_norm_two` | CFT-09-E05 unsolved |
| CFT-09-006 | checkpoint | summary | checkpoint | `Part02.norm_triangle_kernel` | CFT-09-E06 unsolved |
| CFT-10-001 | checkpoint | summary | checkpoint | `Part02.bilinear_pairing_duality` | CFT-10-E01 unsolved |
| CFT-10-002 | checkpoint | summary | checkpoint | `Part02.transpose_coordinate_action` | CFT-10-E02 unsolved |
| CFT-10-003 | unmapped | summary | reexported-proof | `Part02.determinant_top_degree_multiplicative` | CFT-10-E03 unsolved |
| CFT-10-004 | unmapped | summary | reexported-proof | `Part02.alternating_diagonal_volume` | CFT-10-E04 unsolved |
| CFT-10-005 | unmapped | summary | reexported-proof | `Part02.contraction_trace_cyclic` | CFT-10-E05 unsolved |
| CFT-10-006 | unmapped | summary | proved-here | `Part02.wedge_sign_kernel` | CFT-10-E06 unsolved |
| CFT-11-001 | checkpoint | summary | checkpoint | `Part02.gradient_component_contract` | CFT-11-E01 unsolved |
| CFT-11-002 | checkpoint | summary | checkpoint | `Part02.derivative_action_is_jvp` | CFT-11-E02 unsolved |
| CFT-11-003 | checkpoint | summary | checkpoint | `Part02.derivative_pullback_is_vjp` | CFT-11-E03 unsolved |
| CFT-11-004 | checkpoint | summary | checkpoint | `Part02.hessian_vector_action` | CFT-11-E04 unsolved |
| CFT-11-005 | checkpoint | summary | checkpoint | `Part02.jvp_vjp_duality` | CFT-11-E05 unsolved |
| CFT-11-006 | unmapped | summary | proved-here | `Part02.linear_approximation_chain_kernel` | CFT-11-E06 unsolved |
| CFT-12-001 | checkpoint | summary | checkpoint | `Part02.real_inner_complex_coordinates` | CFT-12-E01 unsolved |
| CFT-12-002 | unmapped | summary | reexported-proof | `Part02.area_form_quarter_turn` | CFT-12-E02 unsolved |
| CFT-12-003 | unmapped | summary | reexported-proof | `Part02.oriented_radial_tangent_positive` | CFT-12-E03 unsolved |
| CFT-12-004 | unmapped | summary | reexported-proof | `Part02.oriented_tangent_formula` | CFT-12-E04 unsolved |
| CFT-12-005 | unmapped | summary | reexported-proof | `Part02.boundary_parameter_is_smooth` | CFT-12-E05 unsolved |
| CFT-12-006 | unmapped | summary | reexported-proof | `Part02.boundary_slice_derivative` | CFT-12-E06 unsolved |
| CFT-13-001 | checkpoint | summary | checkpoint | `Part03.max_modulus_on_compact_set` | CFT-13-E01 unsolved |
| CFT-13-002 | unmapped | summary | reexported-proof | `Part03.compact_maximum_exists` | CFT-13-E02 unsolved |
| CFT-13-003 | unmapped | summary | reexported-proof | `Part03.pointwise_norm_le_maximum` | CFT-13-E03 unsolved |
| CFT-13-004 | unmapped | summary | reexported-proof | `Part03.compact_maximum_nonnegative` | CFT-13-E04 unsolved |
| CFT-13-005 | unmapped | summary | reexported-proof | `Part03.compact_maximum_monotone` | CFT-13-E05 unsolved |
| CFT-13-006 | unmapped | summary | reexported-proof | `Part03.outer_maxima_converge` | CFT-13-E06 unsolved |
| CFT-14-001 | checkpoint | summary | checkpoint | `Part03.matrix_power_series` | CFT-14-E01 unsolved |
| CFT-14-002 | unmapped | summary | reexported-proof | `Part03.matrix_power_series_coefficient` | CFT-14-E02 unsolved |
| CFT-14-003 | unmapped | summary | reexported-proof | `Part03.matrix_power_series_radius` | CFT-14-E03 unsolved |
| CFT-14-004 | checkpoint | summary | checkpoint | `Part03.matrix_power_series_sum` | CFT-14-E04 unsolved |
| CFT-14-005 | unmapped | summary | reexported-proof | `Part03.matrix_power_series_analytic` | CFT-14-E05 unsolved |
| CFT-14-006 | unmapped | summary | reexported-proof | `Part03.matrix_power_series_converges` | CFT-14-E06 unsolved |
| CFT-15-001 | checkpoint | summary | checkpoint | `Part03.parametric_boundary_integral` | CFT-15-E01 unsolved |
| CFT-15-002 | unmapped | summary | reexported-proof | `Part03.boundary_integral_continuous` | CFT-15-E02 unsolved |
| CFT-15-003 | checkpoint | summary | checkpoint | `Part03.boundary_integral_is_function_eval` | CFT-15-E03 unsolved |
| CFT-15-004 | checkpoint | summary | checkpoint | `Part03.simple_spectrum_holomorphic_eval` | CFT-15-E04 unsolved |
| CFT-15-005 | unmapped | summary | reexported-proof | `Part03.simple_spectrum_boundary_limit` | CFT-15-E05 unsolved |
| CFT-15-006 | checkpoint | summary | checkpoint | `Part03.simple_spectrum_eval_limit` | CFT-15-E06 unsolved |
| CFT-16-001 | checkpoint | summary | checkpoint | `Part03.holomorphic_matrix_eval` | CFT-16-E01 unsolved |
| CFT-16-002 | unmapped | summary | reexported-proof | `Part03.contour_eval_agrees` | CFT-16-E02 unsolved |
| CFT-16-003 | unmapped | summary | reexported-proof | `Part03.polynomial_compatibility` | CFT-16-E03 unsolved |
| CFT-16-004 | unmapped | summary | reexported-proof | `Part03.locality_on_neighborhood` | CFT-16-E04 unsolved |
| CFT-16-005 | unmapped | summary | reexported-proof | `Part03.functional_calculus_additive` | CFT-16-E05 unsolved |
| CFT-16-006 | unmapped | summary | reexported-proof | `Part03.functional_calculus_multiplicative` | CFT-16-E06 unsolved |
| CFT-17-001 | checkpoint | summary | checkpoint | `Part03.rational_pole_set` | CFT-17-E01 unsolved |
| CFT-17-002 | unmapped | summary | reexported-proof | `Part03.rational_poles_finite` | CFT-17-E02 unsolved |
| CFT-17-003 | unmapped | summary | reexported-proof | `Part03.pole_complement_open` | CFT-17-E03 unsolved |
| CFT-17-004 | checkpoint | summary | checkpoint | `Part03.rational_pole_free_on` | CFT-17-E04 unsolved |
| CFT-17-005 | checkpoint | summary | checkpoint | `Part03.rational_scalar_eval` | CFT-17-E05 unsolved |
| CFT-17-006 | checkpoint | summary | checkpoint | `Part03.rational_matrix_eval` | CFT-17-E06 unsolved |
| CFT-18-001 | checkpoint | summary | checkpoint | `Part03.open_unit_disk` | CFT-18-E01 unsolved |
| CFT-18-002 | checkpoint | summary | checkpoint | `Part03.unit_circle` | CFT-18-E02 unsolved |
| CFT-18-003 | checkpoint | summary | checkpoint | `Part03.sampled_kernel_matrix` | CFT-18-E03 unsolved |
| CFT-18-004 | checkpoint | summary | checkpoint | `Part03.positive_matrix_kernel_on` | CFT-18-E04 unsolved |
| CFT-18-005 | checkpoint | summary | checkpoint | `Part03.matrix_herglotz_kernel` | CFT-18-E05 unsolved |
| CFT-18-006 | unmapped | summary | reexported-proof | `Part03.matrix_herglotz_kernel_positive` | CFT-18-E06 unsolved |
| CFT-19-001 | checkpoint | summary | checkpoint | `Part04.simple_spectrum_approximation` | CFT-19-E01 unsolved |
| CFT-19-002 | unmapped | summary | reexported-proof | `Part04.simple_spectrum_approximation_distinct` | CFT-19-E02 unsolved |
| CFT-19-003 | unmapped | summary | reexported-proof | `Part04.simple_spectrum_approximation_close` | CFT-19-E03 unsolved |
| CFT-19-004 | unmapped | summary | reexported-proof | `Part04.simple_spectrum_approximation_converges` | CFT-19-E04 unsolved |
| CFT-19-005 | unmapped | summary | reexported-proof | `Part04.distinct_spectrum_dense` | CFT-19-E05 unsolved |
| CFT-19-006 | unmapped | summary | reexported-proof | `Part04.diagonal_polynomial_action` | CFT-19-E06 unsolved |
| CFT-20-001 | unmapped | summary | reexported-proof | `Part04.numerical_range_nonempty` | CFT-20-E01 unsolved |
| CFT-20-002 | unmapped | summary | reexported-proof | `Part04.numerical_range_as_sphere_image` | CFT-20-E02 unsolved |
| CFT-20-003 | unmapped | summary | reexported-proof | `Part04.numerical_range_compact` | CFT-20-E03 unsolved |
| CFT-20-004 | unmapped | summary | reexported-proof | `Part04.numerical_range_convex` | CFT-20-E04 unsolved |
| CFT-20-005 | unmapped | summary | reexported-proof | `Part04.numerical_range_perturbation_bound` | CFT-20-E05 unsolved |
| CFT-20-006 | unmapped | summary | reexported-proof | `Part04.spectrum_lies_in_numerical_range` | CFT-20-E06 unsolved |
| CFT-21-001 | unmapped | summary | reexported-proof | `Part04.operator_numerical_range_convex` | CFT-21-E01 unsolved |
| CFT-21-002 | checkpoint | summary | checkpoint | `Part04.closed_operator_numerical_range` | CFT-21-E02 unsolved |
| CFT-21-003 | unmapped | summary | reexported-proof | `Part04.closed_operator_numerical_range_nonempty` | CFT-21-E03 unsolved |
| CFT-21-004 | unmapped | summary | reexported-proof | `Part04.closed_operator_numerical_range_compact` | CFT-21-E04 unsolved |
| CFT-21-005 | checkpoint | summary | checkpoint | `Part04.hilbert_rational_spectral_set_statement` | CFT-21-E05 unsolved |
| CFT-21-006 | checkpoint | summary | checkpoint | `Part04.closed_numerical_range_two_spectral_set` | CFT-21-E06 unsolved |
| CFT-22-001 | checkpoint | summary | checkpoint | `Part04.boundary_positive_map` | CFT-22-E01 unsolved |
| CFT-22-002 | checkpoint | summary | checkpoint | `Part04.boundary_positive_linear_map` | CFT-22-E02 unsolved |
| CFT-22-003 | unmapped | summary | reexported-proof | `Part04.boundary_positive_map_norm` | CFT-22-E03 unsolved |
| CFT-22-004 | unmapped | summary | reexported-proof | `Part04.boundary_positive_map_unital` | CFT-22-E04 unsolved |
| CFT-22-005 | unmapped | summary | reexported-proof | `Part04.boundary_positive_map_star` | CFT-22-E05 unsolved |
| CFT-22-006 | unmapped | summary | reexported-proof | `Part04.boundary_positive_map_preserves_psd` | CFT-22-E06 unsolved |
| CFT-23-001 | checkpoint | summary | checkpoint | `Part04.compressed_adjoint_power` | CFT-23-E01 unsolved |
| CFT-23-002 | unmapped | summary | reexported-proof | `Part04.compressed_adjoint_power_contractive` | CFT-23-E02 unsolved |
| CFT-23-003 | unmapped | summary | reexported-proof | `Part04.doubled_compression_norm_two` | CFT-23-E03 unsolved |
| CFT-23-004 | unmapped | summary | reexported-proof | `Part04.adjacent_power_defect_factorization` | CFT-23-E04 unsolved |
| CFT-23-005 | unmapped | summary | reexported-proof | `Part04.perturbation_commutes` | CFT-23-E05 unsolved |
| CFT-23-006 | unmapped | summary | reexported-proof | `Part04.target_power_bound` | CFT-23-E06 unsolved |
| CFT-24-001 | checkpoint | summary | checkpoint | `Part04.gramian_term` | CFT-24-E01 unsolved |
| CFT-24-002 | unmapped | summary | reexported-proof | `Part04.gramian_term_positive` | CFT-24-E02 unsolved |
| CFT-24-003 | unmapped | summary | reexported-proof | `Part04.gramian_terms_summable` | CFT-24-E03 unsolved |
| CFT-24-004 | checkpoint | summary | checkpoint | `Part04.weighted_gramian` | CFT-24-E04 unsolved |
| CFT-24-005 | unmapped | summary | reexported-proof | `Part04.weighted_gramian_positive` | CFT-24-E05 unsolved |
| CFT-24-006 | unmapped | summary | reexported-proof | `Part04.gramian_difference_positive` | CFT-24-E06 unsolved |

## Ownership

Every remaining chapter already has a Kata issue in the
`crouzeix-textbook-completion` program, listed in the table above. The
whole-book issue `00zs` stays open. `1t51` (Part I checkpoint) is unblocked,
`3ya3` (Chapter 7) and `tdga` (Chapter 8) carry the Part II opening. `8mq3`, `m4qy`, and `q8h9` remain open against their whole-chapter
scope; their accepted core work is recorded in their own comments and Chapters
2 and 3 are not reopened.

Chapters 2 and 3 have no per-chapter Kata issue in this project. They are
complete against the contract, so no issue is created for them here.

## What this inventory does not claim

These counts measure the 216 cards of this textbook. They are not a percentage
of Lax, Spivak, or the Bishop books; that denominator requires the separate
source-item inventory named in the plan's program boundaries. Compiler
acceptance establishes the recorded Lean declarations in the pinned
`leanprover/lean4:v4.32.1` environment and nothing beyond it.
