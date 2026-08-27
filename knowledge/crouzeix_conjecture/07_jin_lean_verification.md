---
id: crouzeix-jin-lean-verification
title: Jin Lean verification audit
type: deep-dive
status: active
created: 2026-08-14
updated: 2026-08-26
tags: [crouzeix-conjecture, lean, formal-verification, source-audit]
confidence: medium
canonical: 07_jin_lean_verification.md
---

# Jin Lean verification audit

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

This chapter distinguishes source-level theorem structure, author-maintained
correspondence records, Harp's local scan observations, the clean-room
builds that did not run because the disk preflight blocked them, and the
separate Harp-local `complete-local` route certification artifacts now
published under `evidence/crouzeix_conjecture/routes/jin/`.

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

Harp's local certification surface keeps the finite-dimensional theorem and
its downstream consequences separate. The Jin route manifest records terminal
declaration `CrouzeixConjecture.crouzeixConjecture` together with consequence
declarations `CrouzeixConjecture.crouzeixRationalBound`,
`CrouzeixConjecture.hilbertSpacePolynomialCrouzeix`, and
`CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet`
([[evidence/crouzeix_conjecture/routes/jin/receipt.json|route receipt]],
[[evidence/crouzeix_conjecture/reviews/jin.json|route review]]).
That is a Harp-local `complete-local` statement about named declarations,
not a claim that the upstream Jin checkout completed a clean-room build.
In reader-facing terms, the current local publication surface covers the
finite-dimensional polynomial theorem, the scalar rational consequence surface,
and the Hilbert-space consequence surface.

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

The Harp-local route certification is stronger on the exact local declaration
surface and narrower on claim ceiling. The published Jin route receipt records
status `passed`, route ID `jin`, allowed axioms
`Classical.choice`, `Quot.sound`, and `propext`, and a `complete-local`
publication state keyed by raw receipt hash
`5931d52d80b2c2c6440900cc85d61cb48169c70e9803f0615ed4ade9b58c4331`
with matching review hash
`a1962f75a98a3ebf73ee33a472ace2c4ff91eead538156f777b8dcf21dcddb4e`
([[evidence/crouzeix_conjecture/routes/jin/receipt.json|receipt]],
[[evidence/crouzeix_conjecture/reviews/jin.json|review]]).

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

## Harp-local route and proof-slice receipts {#harp-local-route-and-proof-slice-receipts}

Harp also maintains a local source-mapped Lean port under the shared
formalization root. The current proof-slice chain is:

| Source-map row | Local declaration | Receipt |
|---|---|---|
| `jin-max-polynomial-modulus` | `CrouzeixConjecture.maxPolynomialModulusOnNumericalRange` | `9e0cf882a8f866a4897a46548c7e387fb0d07b7dee159b3c2260ef9ece1fb563` |
| `jin-polynomial-bound` | `CrouzeixConjecture.PolynomialCrouzeixBound` | `b25ca01a109ad93708d474feb60389e6544f5fb54fe64f557e24571cd7de58c8` |
| `jin-terminal-crouzeix` | `CrouzeixConjecture.crouzeixConjecture` | `a8b6fdc904ed35de4a533b924bf8847c923f3a50e3cc19785ced4bee5f7da0f9` |

These receipts are Harp-local proof checks against
`formalization/lean/Crouzeix/Jin/`, not clean-room builds of the upstream Jin
repository. The receipt chain is validated by the Crouzeix proof-slice tests
and by the shared-root Lean build.

The route-level publication adds the consequence boundary that the older
packet did not yet surface. In the six-row local bundle,
`CrouzeixConjecture.crouzeixConjecture` and
`CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet` are both
certified rows for route `jin`
([[evidence/crouzeix_conjecture/local_formalization/manifest.tsv|bundle manifest]]).
That bundle has six data rows and aggregate SHA-256
`efc8469255219938b958d687d4a62506df937918bd659df18e47eaeb85a71de7`.

The resulting claim ceiling is:

1. upstream Jin source inspection supports the source-level theorem graph and
   revision-specific correspondence discussion;
2. upstream Jin clean-room builds remain blocked by disk preflight; and
3. Harp-local `complete-local` certification now covers the finite-dimensional
   polynomial theorem, scalar rational consequence surface, and Hilbert-space
   consequence surface for the local Jin route declarations named above.

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
