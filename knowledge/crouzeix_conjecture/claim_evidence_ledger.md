---
id: crouzeix-claim-evidence-ledger
title: Crouzeix conjecture claim-evidence ledger
type: claim-ledger
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [crouzeix-conjecture, claims, evidence, provenance]
confidence: medium
canonical: claim_evidence_ledger.md
---

# Crouzeix conjecture claim-evidence ledger

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

The ledger fixes the packet's material claims. A local locator proves that Harp
inspected a particular receipt or observation; it does not inherit the truth of
the upstream theorem.

## CC-001: Crouzeix constant-two conjecture {#cc-001-crouzeix-constant-two-conjecture}

- Class: `SOURCE CLAIM`
- Statement: Every complex matrix satisfies the polynomial numerical-range bound with universal constant two, equivalently its numerical range is a two-spectral set.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [audited v4 receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11)
- Scope: Finite complex matrices and the polynomial statement; rational and Hilbert-space consequences require additional arguments.
- Reproduction: The statement is transcribed and its sharpness is checked separately in CC-002.
- Confidence: `medium`
- Confidence basis: Two pinned candidate-proof artifacts state the same target.
- Caveat: This ledger does not substitute for peer review of either proof.
- Mode: `paraphrase`
- Source stability: `pinned`
- Upstream locator: [Audited v4 main theorem](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L94)

## CC-002: Two-by-two nilpotent sharpness {#cc-002-two-by-two-nilpotent-sharpness}

- Class: `EVIDENCE`
- Statement: The matrix with upper-right entry two and all other entries zero has unit-disk numerical range and norm two, so no smaller universal constant can work.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [audited v4 receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11)
- Scope: Direct two-by-two witness for optimality of the universal constant.
- Reproduction: The packet derives the disk and norm from the displayed matrix.
- Confidence: `high`
- Confidence basis: Elementary finite-dimensional calculation reproduced in the packet.
- Caveat: Sharpness does not prove the upper bound for general matrices.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-003: Crouzeix-Palencia one-plus-square-root-two {#cc-003-crouzeix-palencia-one-plus-square-root-two}

- Class: `SOURCE CLAIM`
- Statement: Crouzeix and Palencia established the universal numerical-range spectral-set constant one plus square root two.
- Source: [CROUZEIX-PALENCIA-2017](source_registry.md#crouzeix-palencia-2017-one-plus-square-root-two-result)
- Locator: [publisher metadata receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L3)
- Scope: Bibliographic and result-level context for the prior universal bound.
- Reproduction: No full-text reproduction was run in Harp.
- Confidence: `medium`
- Confidence basis: Dated DOI metadata and consistent statements in both pinned preprints.
- Caveat: The local receipt is metadata only.
- Mode: `paraphrase`
- Source stability: `dated observation`
- Observed: `2026-08-14`

## CC-004: Symmetrized double-layer identity {#cc-004-symmetrized-double-layer-identity}

- Class: `EVIDENCE`
- Statement: The double-layer setup couples the target and companion through twice the positive map, in the form two Phi of f equals f of A plus alpha of f at A adjointed.
- Source: [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: The identity used in the smoothly bounded convex-domain setup.
- Reproduction: The packet traces the identity into both candidate proofs.
- Confidence: `high`
- Confidence basis: Direct inspection of the pinned TeX and matching Jin construction.
- Caveat: Positivity of the symmetrized value alone does not isolate the target.
- Mode: `paraphrase`
- Source stability: `pinned`
- Upstream locator: [arXiv v1](https://arxiv.org/abs/2608.03841v1)

## CC-005: One-step treatment loses coupling {#cc-005-one-step-treatment-loses-coupling}

- Class: `INFERENCE`
- Statement: Replacing the companion by an unrelated one-step norm bound discards the cross-power structure exploited by both constant-two routes.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [audited v4 receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11)
- Scope: Comparison of the mechanisms in the two pinned preprints, not a theorem about every possible proof.
- Reproduction: Harp follows the Cayley coefficients and the perturbation sequence through their decisive estimates.
- Confidence: `medium`
- Confidence basis: Both derivations use indexed power information after the double-layer identity.
- Caveat: A future proof may reach two without retaining this family.
- Weakens if: Either proof can be reconstructed with only a single isolated companion estimate.
- Falsified by: A verified derivation of the same decisive inequalities from one-step norm data alone.

## CC-010: Jin completion hypotheses {#cc-010-jin-completion-hypotheses}

- Class: `EVIDENCE`
- Statement: Jin's completion theorem assumes an auxiliary simple-spectrum matrix, a same-basis target with diagonal values in the closed disk, an analytic H with H of zero equal to I and positive real part, and a resolvent defect in the adjoint algebra.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [audited v4 receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11)
- Scope: Positive-real completion theorem at the audited revision.
- Reproduction: The packet states every hypothesis before deriving the norm endpoint.
- Confidence: `high`
- Confidence basis: Exact theorem interface in TeX and Lean.
- Caveat: The theorem's proof still depends on the sampled-kernel and Gramian chain.
- Mode: `paraphrase`
- Source stability: `pinned`
- Upstream locator: [completion theorem](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L319)

## CC-011: Auxiliary basis permits repeated target values {#cc-011-auxiliary-basis-permits-repeated-target-values}

- Class: `EVIDENCE`
- Statement: Distinct auxiliary eigenvalues identify the adjoint algebra with diagonal corrections, while the target diagonal values lambda i may repeat.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [formalization-map receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L7)
- Scope: Same-basis completion interface, not a distinct-spectrum assumption on T.
- Reproduction: The packet keeps beta distinct and lambda unrestricted except for modulus.
- Confidence: `high`
- Confidence basis: Explicit in both theorem statement and formalization map.
- Caveat: The auxiliary matrix must still have simple spectrum before the limiting passage.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-012: Herglotz kernel is positive {#cc-012-herglotz-kernel-is-positive}

- Class: `EVIDENCE`
- Statement: An analytic matrix function with positive semidefinite real part has a positive matrix Herglotz kernel.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [audited v4 receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11)
- Scope: Finite sampled kernel positivity on the unit disk.
- Reproduction: The packet records the regularized circle-average proof and its finite-sample consequence.
- Confidence: `high`
- Confidence basis: Direct proof in the pinned manuscript and named Lean theorem.
- Caveat: No broader matrix-valued measure representation is required or claimed.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-013: Origin sample cancels the correction {#cc-013-origin-sample-cancels-the-correction}

- Class: `EVIDENCE`
- Statement: Sampling at half conjugate target values and adding the origin vector minus G inverse P u makes the diagonal correction contribution vanish exactly.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [formalization-map receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L7)
- Scope: Sampled Herglotz-kernel quadratic form in the auxiliary basis.
- Reproduction: The packet computes the parenthesis as Gv plus Pu and substitutes v.
- Confidence: `high`
- Confidence basis: Algebraic identity in manuscript and Lean map.
- Caveat: The cancellation is on the prescribed graph vector, not positivity of the full block for independent vectors.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-014: Ordered pre-Gramian inequality {#cc-014-ordered-pre-gramian-inequality}

- Class: `EVIDENCE`
- Statement: After correction cancellation, sampled kernel positivity gives four Y minus Y G inverse P minus P G inverse Y positive semidefinite, with the displayed noncommuting order.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [audited v4 receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11)
- Scope: Finite-dimensional completion proof before balancing.
- Reproduction: The packet expands the sampled block on v equals minus G inverse P u.
- Confidence: `high`
- Confidence basis: Exact displayed equation in the pinned manuscript.
- Caveat: Reordering P, G inverse, and Y would change the statement.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-015: Gramian anticommutator bounds P-hat {#cc-015-gramian-anticommutator-bounds-p-hat}

- Class: `EVIDENCE`
- Statement: Positivity of Y-hat together with four Y-hat minus Y-hat P-hat minus P-hat Y-hat positive semidefinite forces P-hat at most two I.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [formalization-map receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L7)
- Scope: Balanced weighted-Gramian stage.
- Reproduction: The packet evaluates on an eigenvector of P-hat and uses the first positive term of Y-hat.
- Confidence: `high`
- Confidence basis: Reproduced finite-dimensional eigenvector contradiction.
- Caveat: The proof uses both positivity and the ordered anticommutator inequality.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-016: First nonconstant Gramian term gives norm two {#cc-016-first-nonconstant-gramian-term-gives-norm-two}

- Class: `EVIDENCE`
- Statement: Since P-hat begins with I plus one quarter T-tilde star T-tilde, the bound P-hat at most two I implies the norm of T is at most two.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [audited v4 receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11)
- Scope: Endpoint of the positive-real completion theorem.
- Reproduction: The packet writes the Loewner inequalities and unitary transfer.
- Confidence: `high`
- Confidence basis: Direct consequence of the displayed Gramian series.
- Caveat: Similarity alone does not preserve norm; the balancing and polar-unitary step is required.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-017: Fixed-domain limit precedes outer-domain limit {#cc-017-fixed-domain-limit-precedes-outer-domain-limit}

- Class: `EVIDENCE`
- Statement: Jin first sends simple-spectrum matrices to the target while the outer domain is fixed, then shrinks the outer domains to the numerical range.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [formalization-map receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L7)
- Scope: Passage from the auxiliary finite-dimensional theorem to arbitrary matrices.
- Reproduction: The packet keeps epsilon fixed through the B k limit before sending epsilon down to zero.
- Confidence: `high`
- Confidence basis: Explicit manuscript proof and formalization-map statement.
- Caveat: Reversing the limits would require a uniform argument not supplied here.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-020: Lean exports the polynomial theorem {#cc-020-lean-exports-the-polynomial-theorem}

- Class: `EVIDENCE`
- Statement: The Lean source exports theorem crouzeixConjecture with the finite-dimensional polynomial constant-two conclusion.
- Source: [JIN-REPO-HEAD](source_registry.md#jin-repo-head-pinned-repository-head)
- Locator: [repository archive receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L14)
- Scope: Source-level declaration at the pinned repository head.
- Reproduction: Harp inspected the declaration and its dependency on holomorphicCrouzeixBound.
- Confidence: `high`
- Confidence basis: Direct source inspection.
- Caveat: Harp's clean-room build was blocked before compilation.
- Mode: `paraphrase`
- Source stability: `pinned`
- Upstream locator: [FinalTheorems declaration](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/Lean/CrouzeixConjecture/FinalTheorems.lean#L18)

## CC-021: Lean proposition and proof theorem are distinct {#cc-021-lean-proposition-and-proof-theorem-are-distinct}

- Class: `EVIDENCE`
- Statement: PositiveRealCompletionStatement is a proposition definition, while positiveRealCompletionStatement is the theorem proving that proposition.
- Source: [JIN-REPO-HEAD](source_registry.md#jin-repo-head-pinned-repository-head)
- Locator: [repository archive receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L14)
- Scope: Lean declaration roles and capitalization at repository head.
- Reproduction: Harp inspected both source declarations.
- Confidence: `high`
- Confidence basis: Direct source-level distinction.
- Caveat: Name similarity can obscure the definition-versus-proof boundary.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-022: Lean manuscript digest is stale at repository head {#cc-022-lean-manuscript-digest-is-stale-at-repository-head}

- Class: `EVIDENCE`
- Statement: The manuscript audit identifies the earlier audited v4 bytes, while repository-head v4 has a different byte count and SHA-256.
- Source: [JIN-REPO-HEAD](source_registry.md#jin-repo-head-pinned-repository-head)
- Locator: [head manuscript-audit receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L19)
- Scope: Byte identity and correspondence record, not mathematical divergence.
- Reproduction: Harp compares the audited and head v4 receipt rows.
- Confidence: `high`
- Confidence basis: Distinct measured SHA-256 values.
- Caveat: A stale audit digest does not by itself identify a theorem-level mismatch.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-023: Build observation at audited v4 {#cc-023-build-observation-at-audited-v4}

- Class: `EVIDENCE`
- Statement: The clean-room Lean build at commit 565b6a3 was blocked by the disk preflight before Mathlib cache materialization.
- Source: [HARP-LOCAL-VERIFY](source_registry.md#harp-local-verify-local-build-and-scan-observations)
- Locator: [audited-v4 build log](../../evidence/crouzeix_conjecture/verification/jin-565b6a3-build.log#L1)
- Scope: Harp-local observation on 2026-08-14.
- Reproduction: Re-run acquire.sh with at least the recorded 8 GiB free-space policy.
- Confidence: `high`
- Confidence basis: Hash-bound normalized local log.
- Caveat: Result is blocked, neither passed nor failed.
- Mode: `paraphrase`
- Source stability: `dated observation`
- Observed: `2026-08-14`

## CC-024: Build observation at repository head {#cc-024-build-observation-at-repository-head}

- Class: `EVIDENCE`
- Statement: The clean-room Lean build at commit 9df0783 was blocked by the same disk preflight before Mathlib cache materialization.
- Source: [HARP-LOCAL-VERIFY](source_registry.md#harp-local-verify-local-build-and-scan-observations)
- Locator: [repository-head build log](../../evidence/crouzeix_conjecture/verification/jin-9df0783-build.log#L1)
- Scope: Harp-local observation on 2026-08-14.
- Reproduction: Re-run acquire.sh with at least the recorded 8 GiB free-space policy.
- Confidence: `high`
- Confidence basis: Hash-bound normalized local log.
- Caveat: Result is blocked, neither passed nor failed.
- Mode: `paraphrase`
- Source stability: `dated observation`
- Observed: `2026-08-14`

## CC-025: Commit-scoped prohibited-token scans passed {#cc-025-commit-scoped-prohibited-token-scans-passed}

- Class: `EVIDENCE`
- Statement: The source scans at both pinned Jin revisions completed with zero prohibited-token findings.
- Source: [HARP-LOCAL-VERIFY](source_registry.md#harp-local-verify-local-build-and-scan-observations)
- Locator: [audited scan receipt](../../evidence/crouzeix_conjecture/verification_manifest.tsv#L3) and [head scan receipt](../../evidence/crouzeix_conjecture/verification_manifest.tsv#L5)
- Scope: The prohibited token classes implemented by acquire.sh over tracked sources.
- Reproduction: Re-run the revision-bound source-scan operations.
- Confidence: `high`
- Confidence basis: Empty normalized logs with manifest-bound hashes and zero exits.
- Caveat: Token scanning is not type checking, axiom auditing, or mathematical verification.
- Mode: `paraphrase`
- Source stability: `dated observation`
- Observed: `2026-08-14`

## CC-030: Lorist-Schwenninger perturbation lemma {#cc-030-lorist-schwenninger-perturbation-lemma}

- Class: `SOURCE CLAIM`
- Statement: Uniformly bounded commuting perturbations of a compressed contraction-power 2-dilation force the finite-dimensional operator norm to be at most two.
- Source: [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: Lemma 1 for finite-dimensional Hilbert space.
- Reproduction: The packet derives the recurrence and contradiction from the stated hypotheses.
- Confidence: `medium`
- Confidence basis: Pinned source plus a line-by-line Harp derivation.
- Caveat: General Hilbert spaces need norm attainment or another limiting argument.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-031: Lorist-Schwenninger recurrence {#cc-031-lorist-schwenninger-recurrence}

- Class: `EVIDENCE`
- Statement: Completing the square yields kappa m n minus m n plus one at least r n, whose iteration gives the displayed weighted lower bound for m one.
- Source: [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: Scalar recurrence along a top singular vector when kappa exceeds one.
- Reproduction: The packet defines S n, m n, y n, and r n and repeats the iteration.
- Confidence: `high`
- Confidence basis: Direct algebra from pinned equations.
- Caveat: The final contradiction also needs the separate estimate involving Q star V T x.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-032: Equation one uniformly bounds E-n T-n {#cc-032-equation-one-uniformly-bounds-en-tn}

- Class: `INFERENCE`
- Statement: If the E n are bounded by M, Equation 1 implies the T powers are bounded by 2 plus M and hence E n T n is bounded by M times 2 plus M.
- Source: [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: The limiting justification in Lemma 1.
- Reproduction: Harp applies the triangle inequality, contraction of Q, and isometry of V.
- Confidence: `high`
- Confidence basis: Two-line norm estimate from Equation 1.
- Caveat: The estimate depends on the uniform bound M from the lemma hypothesis.
- Weakens if: Equation 1 is interpreted without V isometric or Q contractive.
- Falsified by: A counterexample satisfying all lemma hypotheses with unbounded E n T n.

## CC-033: Double-layer realizes the perturbation family {#cc-033-double-layer-realizes-the-perturbation-family}

- Class: `EVIDENCE`
- Statement: In the numerical-range application, V is built from the positive boundary density, Q multiplies by f, and E n equals alpha of f to the n evaluated at A.
- Source: [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: Smoothly bounded convex outer domain and normalized f.
- Reproduction: The packet checks V star Q n V equals Phi of f n and substitutes the double-layer identity.
- Confidence: `high`
- Confidence basis: Explicit source equations.
- Caveat: The outer-domain and finite-dimensional reductions are inherited from the cited standard setup.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-034: Functional calculus independently bounds E-n T-n {#cc-034-functional-calculus-independently-bounds-en-tn}

- Class: `INFERENCE`
- Statement: In the application, E n T n equals the functional calculus of alpha of f n times f n, so boundedness also follows directly from bounded alpha and the homomorphism.
- Source: [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: Application-level route, separate from the general lemma estimate in CC-032.
- Reproduction: Harp uses commutativity and multiplicativity of the functional calculus.
- Confidence: `medium`
- Confidence basis: Direct composition of the stated maps.
- Caveat: This route uses application structure absent from the abstract perturbation lemma.
- Weakens if: Alpha or the functional-calculus homomorphism is not bounded on the stated algebra.
- Falsified by: A normalized application satisfying the source setup with unbounded alpha of f n times f n under the calculus.

## CC-035: Lorist-Schwenninger rational two-spectral-set conclusion {#cc-035-lorist-schwenninger-rational-two-spectral-set-conclusion}

- Class: `SOURCE CLAIM`
- Statement: The Lorist-Schwenninger theorem concludes the constant-two bound for rational functions with poles off the closure of the numerical range.
- Source: [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: Bounded operators after the standard reductions stated by the source.
- Reproduction: Harp reconstructs the finite-dimensional core but does not independently rebuild every cited reduction.
- Confidence: `medium`
- Confidence basis: Pinned theorem statement and derivation.
- Caveat: Preprint status and inherited standard machinery remain part of the review boundary.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-036: Lorist-Schwenninger abstract uniform-algebra variant {#cc-036-lorist-schwenninger-abstract-uniform-algebra-variant}

- Class: `SOURCE CLAIM`
- Statement: The source states an abstract variant for a commutative uniform algebra, unital bounded antilinear alpha, bounded homomorphism theta, and completely positive symmetrized map, concluding norm theta at most two.
- Source: [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: Remark-level abstract extension using Arveson and Stinespring.
- Reproduction: Harp records the interface but does not formalize the extension theorems.
- Confidence: `medium`
- Confidence basis: Explicit statement and proof sketch in pinned TeX.
- Caveat: This scalar norm conclusion is not the completely bounded Crouzeix conjecture.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-037: Neither proof directly yields the completely bounded case {#cc-037-neither-proof-directly-yields-the-completely-bounded-case}

- Class: `SOURCE CLAIM`
- Statement: Lorist and Schwenninger state that neither their proof nor Jin's directly derives the completely bounded case; their commutation hypothesis fails under matrix amplification.
- Source: [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: Direct applicability of the two proof mechanisms to matrix amplification.
- Reproduction: The packet identifies the commutativity and scalar-kernel boundaries.
- Confidence: `medium`
- Confidence basis: Explicit limitation in pinned arXiv v1.
- Caveat: It does not prove that no future adaptation can handle amplification.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-040: Both proofs retain a complete power family {#cc-040-both-proofs-retain-a-complete-power-family}

- Class: `INFERENCE`
- Statement: Jin packages all powers in a Cayley family, while Lorist-Schwenninger retain them as a perturbation sequence; both use cross-power relations before taking the norm endpoint.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript) and [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Jin v4 receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11) and [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: Mechanism-level comparison, not source terminology or theorem equivalence.
- Reproduction: The packet traces power coefficients and adjacent-power recurrence separately.
- Confidence: `medium`
- Confidence basis: Direct structural comparison of both pinned derivations.
- Caveat: The finite-dimensional lemmas and hypotheses remain different.
- Weakens if: Either decisive proof step can be derived without its power-indexed family.
- Falsified by: A source-faithful reconstruction showing that one proof uses only a single isolated f value.

## CC-041: Lorist-Schwenninger call Jin independent and different {#cc-041-lorist-schwenninger-call-jin-independent-and-different}

- Class: `EVIDENCE`
- Statement: Lorist and Schwenninger report that Jin's proof appeared independently and uses a different function-theoretic approach.
- Source: [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: The authors' historical characterization in arXiv v1.
- Reproduction: Direct source inspection.
- Confidence: `medium`
- Confidence basis: Explicit sentence in pinned TeX.
- Caveat: Harp does not independently reconstruct the private discovery timeline.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-042: Jin attributes sampling and cancellation to ChatGPT {#cc-042-jin-attributes-sampling-and-cancellation-to-chatgpt}

- Class: `EVIDENCE`
- Statement: Jin's audited manuscript attributes the scaled conjugate-eigenvalue samples and compensating origin sample to OpenAI ChatGPT.
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [audited v4 receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11)
- Scope: Author disclosure about one proof-development idea.
- Reproduction: Harp inspected the disclosure and the cited equations.
- Confidence: `medium`
- Confidence basis: Explicit pinned disclosure.
- Caveat: No transcript establishes the prompting, intermediate reasoning, or independent novelty of each substep.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-043: Lorist-Schwenninger disclose ChatGPT strategy exploration {#cc-043-lorist-schwenninger-disclose-chatgpt-strategy-exploration}

- Class: `EVIDENCE`
- Statement: Lorist and Schwenninger disclose using ChatGPT 5.6 Pro to explore proof strategies while retaining author responsibility.
- Source: [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Lorist-Schwenninger TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)
- Scope: Author disclosure in arXiv v1.
- Reproduction: Direct source inspection.
- Confidence: `medium`
- Confidence basis: Explicit pinned disclosure.
- Caveat: The disclosure does not identify prompts, rejected routes, or which strategy fragments affected the final lemma.
- Mode: `paraphrase`
- Source stability: `pinned`

## CC-044: Underlying agent transcripts are absent {#cc-044-underlying-agent-transcripts-are-absent}

- Class: `MISSING`
- Statement: The inspected corpus lacks the underlying agent transcripts, per-agent prompts, complete approach registry, rejected-route history, token use, and elapsed-time accounting.
- Source: [JIN-REPO-HEAD](source_registry.md#jin-repo-head-pinned-repository-head) and [LS-ARXIV-V1](source_registry.md#ls-arxiv-v1-lorist-schwenninger-arxiv-v1)
- Locator: [Jin prompt receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L20) and [Lorist-Schwenninger source receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L27)
- Scope: Process evidence for AI-assisted discovery.
- Reproduction: Harp inventories the public pinned artifacts only.
- Confidence: `high`
- Confidence basis: Exact source rosters contain disclosures and one task prompt but no run archive.
- Caveat: Absence from the inspected corpus is not proof that private records do not exist.
- Resolves when: A provenance-preserving transcript bundle binds prompts, models, tool calls, outputs, timestamps, and selection decisions to the published artifacts.

## CC-045: Preprints manuscript bytes are not mapped to Git {#cc-045-preprints-manuscript-bytes-are-not-mapped-to-git}

- Class: `MISSING`
- Statement: Harp could not acquire the Preprints.org manuscript bytes and therefore cannot map that posting to any Jin Git revision.
- Source: [JIN-PREPRINTS-V1](source_registry.md#jin-preprints-v1-preprints-org-metadata-record)
- Locator: [provenance 403 record](../../evidence/crouzeix_conjecture/PROVENANCE.md#L17)
- Scope: Byte identity of the externally posted preprint.
- Reproduction: Retry identity-preserving acquisition without bypassing access controls.
- Confidence: `high`
- Confidence basis: Explicit HTTP 403 acquisition outcome and metadata-only receipt.
- Caveat: The posting may match one Git artifact; Harp lacks the bytes needed to establish that.
- Resolves when: Versioned posted bytes are acquired lawfully, hashed, and compared byte-for-byte with the registered Git manuscripts.

## CC-046: Annals submission status is not established {#cc-046-annals-submission-status-is-not-established}

- Class: `MISSING`
- Statement: The repository contains an Annals-formatted manuscript and calls it a submission manuscript, but the inspected evidence does not establish an actual submission, review, or editorial state.
- Source: [JIN-ANNMATH](source_registry.md#jin-annmath-annals-formatted-manuscript)
- Locator: [Annals-formatted TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L13)
- Scope: Venue workflow status only.
- Reproduction: Inspect an author- or venue-issued submission receipt bound to the manuscript.
- Confidence: `high`
- Confidence basis: Artifact presence is recorded; external workflow evidence is absent.
- Caveat: This claim does not dispute that the source was formatted for the venue.
- Resolves when: A verifiable submission or editorial receipt identifies the venue, date, manuscript, and status.

Back to the [Crouzeix conjecture index](crouzeix_conjecture_index.md).
