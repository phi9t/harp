# Harp theorem assurance specification

Status: approved for implementation. Parent: [mathematics program](2026-09-07-harp-mathematics-program-design.md).

## Deliverables

Create a canonical review manuscript at
`knowledge/crouzeix_textbook/harp_mathematical_audit.md`. Link it from the book
index and Chapter 36, register its managed-document identity through existing
owners, and regenerate the corpus and export. Keep the chapter roster at 36.

Create the execution record at
`docs/workstream/harp-mathematics/theorem-assurance-review.md`, containing a
revision-pinned checklist, compiler evidence references, findings, and reviewer
disposition. Technical mathematical exposition belongs in the canonical
manuscript, not in a second competing workstream document.

Create a focused formal audit module at
`formalization/lean/CrouzeixTextbook/HarpStatementAudit.lean`. It states an
independent, expanded textbook proposition and proves its equivalence to the
maintained `MainTheoremStatement`. Import it from `CrouzeixTextbook.lean` so the
normal textbook target checks it. This equivalence is a semantic inspection
aid, not an independent proof of the terminal inequality.

## Exact intended mathematical scope

For every nonempty finite complex matrix index type, every square matrix A,
and every scalar complex polynomial p, the target is

$$
\|p(A)\|_{2\to2}\le 2\sup_{z\in W(A)}|p(z)|,
\qquad W(A)=\{x^*Ax:\|x\|_2=1\}.
$$

The audit must expand the right-hand supremum, matrix polynomial evaluation,
and the induced Euclidean operator norm. It must prove numerical-range
nonemptiness and boundedness of the scalar image before interpreting the Lean
`sSup` as the mathematical supremum. Inspect compactness if calling it a maximum.
State the positive-dimension restriction rather than concealing empty-index
behavior. Distinguish finite matrices from the separately transported
Hilbert-space and rational-functional-calculus statements.

## Source inspection requirements

Trace the following actual owners, including their transitive dependencies:

- `formalization/lean/CrouzeixConjecture/Euclidean.lean`: matrix/vector
  representation and norm correspondence.
- `formalization/lean/CrouzeixConjecture/Definitions.lean`: numerical range,
  positivity, and polynomial evaluation.
- `formalization/lean/CrouzeixConjecture/Statements.lean`: supremum and universal
  terminal quantifiers.
- `formalization/lean/Crouzeix/Harp/MainTheorem.lean`: fixed-domain construction,
  normalization, and limits.
- `formalization/lean/Crouzeix/Harp/Consequences.lean`: what the adapters add.
- The positive cubature, atomic L2 dilation, finite recurrence, and perturbation
  modules under `formalization/lean/Crouzeix/Harp/`.

The named audit proposition is proposed as `HarpStatementAudit.textbookBound`;
its exact Lean signature is frozen only after inspecting the current norm
instances and elaborating the expanded definition. Its statement must not be
an alias whose only content is `MainTheoremStatement`.

## Seven required mathematical checks

1. Norm semantics: demonstrate the induced Euclidean norm on a nonnormal
   two-by-two matrix and distinguish it from entrywise or Frobenius norms.
2. Positive cubature: positive scalar weights sum to the measure mass; absorbed
   positive matrix weights sum to 2I. These are different normalizations.
3. Dilation: verify the square-root embedding factor, adjoint orientation,
   isometry, contraction, and moment identities through N+1.
4. Quantifiers: fix T, the commuting perturbation family, and its uniform bound
   before choosing a witness for each N. Do not assume a common dilation space.
5. Recurrence: retain the terminal moment, prove denominator positivity, and
   check the direction of replacing the horizon-dependent squared displacement
   by a common upper bound.
6. Limits: take the horizon limit for fixed normalized data, then the matrix
   approximation limit at fixed outer domain, then the domain limit. Explain
   the zero-normalization branch separately.
7. Provenance: distinguish Harp's finite-cubature construction from shared LS
   scalar, norm-attainment, and dilation support. Excluding an LS terminal
   theorem does not imply independence of all LS mathematics.

For each check, give the ordinary-language claim, complete mathematical
statement, source declaration, hypotheses used, proof dependencies, and verdict.
Record gaps precisely. An audit failure is not repaired by weakening the
intended proposition or silently adding hypotheses.

## Boundary and adversarial checks

Include A=0, constant p, one-dimensional matrices, a nonzero nilpotent matrix,
and p vanishing on the numerical range. The last case must follow the actual
zero-normalization argument, not division by zero or an unjustified assertion
that every defective matrix is diagonalizable.

Compile deliberately mismatched audit fixtures in temporary modules and verify
that they do not pass as equivalent statements: coefficient norm substituted
for operator norm, a missing unit-vector condition, swapped quantifiers, and
an additional unproved terminal premise. Some mismatches may still state true
mathematics; the check is exact correspondence, not falsity of every mutation.

Use existing axiom and provider-closure validators. Require no `sorryAx`,
unapproved project axioms, or import of another terminal proof as a shortcut.
Record standard Lean axioms accurately rather than describing all classical
choice as a hidden project axiom.

## Acceptance states

- `audit-prepared`: manuscript and complete inspection checklist exist.
- `locally-audited`: expanded statement comparison compiles, all seven checks
  have source-backed dispositions, and agent mathematical and correspondence
  reviews have no unresolved blocking finding.
- `externally-reviewed`: an identified independent human operator theorist
  reviewed the pinned manuscript and their actual report is retained with
  permission. This is not achievable merely by agent review or compilation.

The local workstream may finish at `locally-audited` while external review is
explicitly pending. No contact, attribution, or external-review claim is made
without the user's authorization and the reviewer's actual participation.
