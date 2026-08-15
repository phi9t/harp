---
id: crouzeix-jin-lean-verification
title: Jin Lean verification audit
type: deep-dive
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [crouzeix-conjecture, lean, formal-verification, source-audit]
confidence: medium
canonical: 07_jin_lean_verification.md
---

# Jin Lean verification audit

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

This chapter distinguishes source-level theorem structure, author-maintained
correspondence records, Harp's local scan observations, and the clean-room
builds that did not run because the disk preflight blocked them.

## Revision identities {#revision-identities}

The formalization-matched v4 manuscript is pinned at
`565b6a3e0659b6e0785f783b016c3f6d9f171fa5`. Its TeX has SHA-256
`5713de029c4a7486e25e86d16e6413d04929bdf5f92439c3237d4a930b1c9242`.

The inspected repository head is
`9df07838327b988e3924453daa29c8cd726d34b0`. Its later v4 TeX has SHA-256
`27f77d75a02faa39a613e18a6b52f3548195b73ceee4996ced38f54319a00315`.

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-022: Lean manuscript digest is stale at repository head|EVIDENCE - CC-022]].**
The checked-in `Lean/MANUSCRIPT_AUDIT.md` remains byte-bound to the earlier
audited manuscript. The different digest at repository head means the
correspondence claim must remain revision-specific. It does not by itself show
that any theorem changed or became false.

## Proposition definition versus proving theorem {#proposition-definition-versus-proving-theorem}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-021: Lean proposition and proof theorem are distinct|EVIDENCE - CC-021]].**
The capitalization boundary is semantically important:

```text
def PositiveRealCompletionStatement : Prop := ...

theorem positiveRealCompletionStatement :
    PositiveRealCompletionStatement := by ...
```

`PositiveRealCompletionStatement` defines the proposition interface:

- auxiliary simple diagonalization;
- same-basis target diagonal values;
- modulus at most one;
- `IsPositiveRealCompletion`;
- norm conclusion at most two.

`positiveRealCompletionStatement` proves that interface by invoking the
kernel model, sampled positivity, Gramian, square-root, and polar-decomposition
declarations in the preceding modules. A `#check` of the proposition name would
not prove that the proposition has an inhabitant; the lowercase theorem is the
proof-bearing declaration.

## Theorem and consequence graphs {#theorem-and-consequence-graphs}

At the source-interface level, the principal dependency chain is:

```text
crouzeixConjecture
  <- holomorphicCrouzeixBound
  <- fixed-domain double-layer bound
  <- positiveRealCompletionStatement
  <- PositiveRealCompletionStatement
```

This arrow notation means “depends on,” not definitional equality. The
completion proposition is the contract; its lowercase theorem discharges it;
the fixed-domain route builds an admissible completion; the outer-limit theorem
passes to the target matrix; and `crouzeixConjecture` specializes the
holomorphic result to polynomials.

The exported consequence graph also contains:

```text
crouzeixRationalSpectralSetCorollary
crouzeixRationalBound
crouzeixConstantTwo_isLeast_finTwo
```

The rational declarations specialize the holomorphic result under pole
freeness. The least-constant declaration uses the finite two-dimensional
sharpness witness.

## Exported polynomial theorem {#exported-polynomial-theorem}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-020: Lean exports the polynomial theorem|EVIDENCE - CC-020]].**
At repository head, `FinalTheorems.lean` exports:

```text
theorem crouzeixConjecture (A : SquareMatrix n) (p : Polynomial C) :
  norm (polynomialEval p A) <=
    2 * maxPolynomialModulusOnNumericalRange A p
```

Its body calls `holomorphicCrouzeixBound`, rewrites holomorphic evaluation to
polynomial evaluation, and simplifies the maximum-modulus definition. This is
a direct source observation. Harp did not complete an independent build of the
declaration.

## Formalized mechanism map {#formalized-mechanism-map}

The repository's formalization map names declarations for the major proof
obligations:

| Manuscript obligation | Named Lean boundary |
|---|---|
| Repeated samples and target values | `finite_type_sampling_quadratic_nonneg`, `completionSamplePoint` |
| Diagonal correction from `generatedAlgebra Bᴴ` | `exists_diagonal_correction_of_mem_generatedAlgebra_conjTranspose` |
| Exact compensating vector | `completion_mulVec_add_eq_zero` |
| Correction cancellation | `completionCorrectionKernel_sampling_eq_zero` |
| Ordered pre-Gramian inequality | `completion_X_inequality_of_positiveKernel` |
| Two weighted Gramians | `completionP_congruence_eq_gramian_four`, `completionR_congruence_eq_gramian_two` |
| Eigenvalue and first-term endpoint | `gramian_four_eigenvalues_le_two`, `norm_le_two_of_gramian_inequality` |
| Fixed-domain simple-spectrum limit | `tendsto_simpleSpectrumHolomorphicEval_of_differentiableOn_neighborhood` |
| Outer-domain limit | `tendsto_maxFunctionModulusOnSet_of_outerApproximation` |

This mapping is useful review structure, but the map file is author-maintained
metadata. A successful build plus source inspection is needed to turn every
row into an independently observed compilation result.

## Local build observations {#local-build-observations}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-023: Build observation at audited v4|EVIDENCE - CC-023]].**
At `565b6a3`, acquisition verified the Git identity and prepared the pinned
toolchain, but the build operation stopped at the typed preflight:

```text
result: blocked
reason: insufficient-disk-for-pinned-mathlib-cache
available: 4,958,564 KiB
required: 8,388,608 KiB
```

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-024: Build observation at repository head|EVIDENCE - CC-024]].**
The same preflight blocked the clean-room build at `9df0783`. Neither receipt
is a compile failure. Neither is a passing build.

The build receipts bind:

- exact source commit and tree;
- exact NUL-delimited command digest;
- acquisition-script digest;
- Lean `v4.28.0`;
- pinned Mathlib revision; and
- normalized local log digest.

## Prohibited-token scans {#prohibited-token-scans}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-025: Commit-scoped prohibited-token scans passed|EVIDENCE - CC-025]].**
Both revision-scoped source scans exited zero with empty normalized logs. Under
the scanner implemented by `acquire.sh`, no prohibited tokens were found.

That observation is deliberately narrow. A token scan does not establish:

- that Lean elaborates the sources;
- that every theorem depends only on accepted axioms;
- that the formal statement matches every prose theorem;
- that numerical definitions use the intended norm in every layer; or
- that the mathematics is correct outside the formalized statement.

## Axiom and trust boundary {#axiom-and-trust-boundary}

The repository includes `AxiomAudit.lean` and an author-maintained
`AXIOM_AUDIT.md`. Their source declares an audit across the completion,
double-layer, outer-limit, polynomial, rational, and Hilbert-space layers.
Without a completed Harp build, the strongest local statement is that these
audit artifacts exist at the pinned revisions and their bytes are receipt-bound.

Even a successful `#print axioms` result would trust Lean's kernel, Mathlib, the
formal definitions, the compiler/runtime used to build them, and the
correspondence between formal statements and the mathematical claim. Formal
verification narrows proof-checking risk; it does not remove the source-model
boundary.

## What would close the local verification gap {#what-would-close-the-local-verification-gap}

1. Provide at least the policy-required disk headroom.
2. Re-run `evidence/crouzeix_conjecture/acquire.sh`.
3. Preserve the exact toolchain, Mathlib revision, source commits, and command
   digests.
4. Record passing or failing build logs separately from source scans.
5. Review the exported statements and axiom output against the exact manuscript
   digest being claimed.

Back to the [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Crouzeix conjecture index]].
