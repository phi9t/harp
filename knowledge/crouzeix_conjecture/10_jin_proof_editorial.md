---
id: crouzeix-jin-proof-editorial
title: Crouzeix, Jin, and why the proof is an engineering object
type: editorial
status: active
created: 2026-08-20
updated: 2026-08-20
tags: [crouzeix-conjecture, jin-proof, lean, mathematical-engineering, frontier-lab]
confidence: medium
canonical: 10_jin_proof_editorial.md
---

# Crouzeix, Jin, and why the proof is an engineering object

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

The Crouzeix conjecture is easy to state in the same way that the best ML
systems problems are easy to state: the difficulty is not in parsing the
interface, but in making every hidden coupling survive the route to a usable
bound.

For a complex matrix $A$, its numerical range is

$$
W(A)=\{x^*Ax:\|x\|=1\}.
$$

The conjecture says that for every polynomial $p$,

$$
\|p(A)\|\le 2\max_{z\in W(A)}|p(z)|.
$$

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-001-crouzeix-constant-two-conjecture|SOURCE CLAIM - CC-001]].**
The constant cannot be improved: the elementary two-by-two nilpotent example
hits the factor two exactly.
**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-002-two-by-two-nilpotent-sharpness|EVIDENCE - CC-002]].**

For a frontier-lab research engineer, the right mental model is not
"decorative functional analysis." It is an operator-norm generalization bound.
The input is a scalar control region, $W(A)$; the output is a matrix-function
control statement, $\|p(A)\|$. The conjecture asks whether the finite matrix
operator can amplify scalar behavior by more than a universal factor of two.
That is exactly the kind of question that appears whenever a local scalar
certificate is used to justify a global high-dimensional transformation.

## Why the old route stalled {#why-the-old-route-stalled}

The pre-2026 landscape already had a powerful double-layer calculus. In the
packet's notation, it gives a positive symmetrized expression

$$
2\Phi(f)=f(A)+\alpha(f)(A)^*.
$$

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-004-symmetrized-double-layer-identity|EVIDENCE - CC-004]].**
That is a real structural handle: positivity is now available. The problem is
that positivity applies to a coupled object, while the target theorem only
wants $f(A)$. If the companion term is bounded independently, the argument
loses the correlation that made the identity sharp. That kind of failure
should be familiar from systems work: replacing a joint invariant by two
separate worst-case bounds usually spends the margin before the decisive step.

The Crouzeix-Palencia result reaches the universal constant
$1+\sqrt2$.
**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-003-crouzeix-palencia-one-plus-square-root-two|SOURCE CLAIM - CC-003]].**
The candidate constant-two proofs are interesting because they do not merely
tighten an estimate. They keep a family of related constraints alive until the
point where the companion can be eliminated or made harmless. Harp calls this
the "shared power family" framing. It is an inference over the two candidate
proofs, not a term claimed by either source.
**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-005-one-step-treatment-loses-coupling|INFERENCE - CC-005]].**

## Jin's move {#jins-move}

Jin's route turns the single double-layer identity into a positive-real family.
After normalizing $f$ on a fixed outer domain, it applies the Cayley transform

$$
c_w(t)=\frac{1+w f(t)}{1-w f(t)}
=1+2\sum_{m=1}^{\infty}w^m f(t)^m
$$

and then pushes that family through the positive double-layer map. The result
is an analytic matrix function $H(w)$ with $H(0)=I$ and positive semidefinite
real part. In the right auxiliary eigenbasis, the difference between $H(w)$
and the resolvent $(I-wT)^{-1}$ lies in a diagonal adjoint algebra.

The proof is not saying that the diagonal correction is small. It is saying
that the correction has the exact algebraic shape needed to be canceled.
That distinction is the whole point.

The finite-dimensional interface is the positive-real completion theorem.
It assumes:

- an auxiliary simple-spectrum matrix, used to expose a diagonal correction
  algebra;
- a same-basis target $T$ with diagonal values $\lambda_i$ in the closed unit
  disk;
- an analytic positive-real function $H$ normalized by $H(0)=I$; and
- a resolvent defect
  $H(w)-(I-wT)^{-1}$ in the adjoint algebra.

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-010-jin-completion-hypotheses|EVIDENCE - CC-010]].**
The simple-spectrum assumption is on the auxiliary matrix, not on the target
values $\lambda_i$. Repeated target values are allowed.
**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-011-auxiliary-basis-permits-repeated-target-values|EVIDENCE - CC-011]].**

## The cancellation as a proof primitive {#the-cancellation-as-a-proof-primitive}

The central device is a sampled Herglotz-kernel inequality. If $F$ is analytic
with positive semidefinite real part, then

$$
\mathcal L_F(w,z)=\frac{F(w)+F(z)^*}{1-w\overline z}
$$

is a positive matrix kernel.
**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-012-herglotz-kernel-is-positive|EVIDENCE - CC-012]].**
Jin samples this kernel at

$$
w_i=\overline{\lambda_i}/2
$$

and adds a sample at the origin with a vector chosen to satisfy

$$
Gv+Pu=0.
$$

That equation makes the correction term vanish exactly.
**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-013-origin-sample-cancels-the-correction|EVIDENCE - CC-013]].**
This is the proof's decisive engineering move: turn an unknown term into a
structured interface, then choose the test vector that routes around it.

After cancellation, the positive kernel yields the ordered inequality

$$
4Y-YG^{-1}P-PG^{-1}Y\succeq0.
$$

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-014-ordered-pre-gramian-inequality|EVIDENCE - CC-014]].**
The order matters. Treating this like a scalar inequality or silently
commuting the factors changes the claim. This is one reason the proof is a good
formalization target: it contains exactly the sort of noncommutative book-
keeping that looks harmless in prose and becomes unforgiving in Lean.

## Why the constant two appears {#why-the-constant-two-appears}

The remaining argument balances by $G^{1/2}$ and reveals two weighted Gramians:

$$
\widehat P=
\sum_{k=0}^{\infty}4^{-k}\widetilde T^{*k}\widetilde T^k,
\qquad
\widehat Q=
\sum_{k=0}^{\infty}2^{-k}\widetilde T^{*k}\widetilde T^k.
$$

The ordered inequality becomes

$$
4\widehat Y-\widehat Y\widehat P-\widehat P\widehat Y\succeq0,
\qquad
\widehat Y=\widehat Q-\widehat P.
$$

This forces $\widehat P\preceq2I$.
**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-015-gramian-anticommutator-bounds-p-hat|EVIDENCE - CC-015]].**
The first nonconstant term of $\widehat P$ is
$\widetilde T^*\widetilde T/4$, so the endpoint is
$\|\widetilde T\|\le2$. The final transfer back to $T$ uses the polar
decomposition of the auxiliary basis. Similarity alone would not preserve the
operator norm.
**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-016-first-nonconstant-gramian-term-gives-norm-two|EVIDENCE - CC-016]].**

This is a useful proof-reading checksum. If a summary of Jin's proof does not
mention the origin sample, the ordered anticommutator, and the polar-unitary
endpoint, it is probably describing the shape of the proof without the load-
bearing machinery.

## From the finite lemma to the conjecture {#from-the-finite-lemma-to-the-conjecture}

The completion theorem is the finite-dimensional core. To get the matrix
theorem, Jin holds an outer domain fixed, approximates by simple-spectrum
matrices inside that domain, passes to the target matrix, and only then shrinks
the outer domains back to $W(A)$.
**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#cc-017-fixed-domain-limit-precedes-outer-domain-limit|EVIDENCE - CC-017]].**

That limit order matters. It avoids asking a moving boundary and a moving
matrix to share an unstated uniform estimate. In research-engineering terms,
it separates two convergence contracts instead of mixing them into one
unreviewable step.

Polynomials are then the easy specialization because they are holomorphic
everywhere. In Harp's Lean port, the shared terminal assembly is factored as
`polynomialCrouzeixBound_of_holomorphicCrouzeixBound`, which applies the
holomorphic bound on the universal domain and rewrites holomorphic polynomial
evaluation to `polynomialEval`. The source is
`formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean`,
starting at the declaration for
`polynomialCrouzeixBound_of_holomorphicCrouzeixBound`.
The Jin terminal surface delegates to that shared theorem
in `formalization/lean/Crouzeix/Jin/Terminal.lean`, and the public final
theorem uses the same assembly point in
`formalization/lean/CrouzeixConjecture/FinalTheorems.lean`.

## What Lean changed about the proof conversation {#what-lean-changed-about-the-proof-conversation}

The social question "is the proof done?" becomes several sharper questions:

1. Does the mathematical source state the target theorem?
2. Does the source-to-formal target map identify the statement being checked?
3. Does the local Lean development compile the target declaration without
   forbidden proof shortcuts?
4. Does the theorem's axiom audit stay within the expected foundations?
5. Does the prose article avoid claiming more than those receipts show?

Harp's formal target lock binds the terminal target to Jin's audited commit
`565b6a3e0659b6e0785f783b016c3f6d9f171fa5`, its source tree, the expected
declaration `CrouzeixConjecture.crouzeixConjecture`, and the statement digest
([formal_target.lock.json](../../labs/crouzeix_proof_reproduction/formal_target.lock.json#L49)).
The terminal proof-slice receipt records that the expected Lean declaration
passed and that its axiom audit passed
([receipt.json](../../labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-terminal-crouzeix/attempt-001/receipt.json#L1)).
The supporting polynomial-bound and maximum-modulus slices also have passing
receipts
([polynomial receipt](../../labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-polynomial-bound/attempt-001/receipt.json#L1),
[maximum-modulus receipt](../../labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-004/receipt.json#L1)).

This does not mean every surrounding mathematical or publication question is
settled. It means the proof discussion now has a machine-checkable spine:
there is a named theorem, a locked statement identity, a compiled terminal
assembly, and receipts for the proof slices Harp claims to have run.

## The status boundary {#the-status-boundary}

The current strongest Harp-local statement is narrow and valuable:

> Harp has a source-mapped, Lean-checked terminal assembly for Jin's polynomial
> Crouzeix route, with passing receipts for the tracked proof slices and a
> shared terminal theorem factored through the holomorphic route.

That statement is not the same as saying that the external mathematical
community has completed review, that the Preprints.org bytes were locally
acquired, or that every revision of Jin's repository has the same manuscript-
to-Lean correspondence. Those are different evidence channels. The packet's
critical assessment keeps those boundaries explicit, including the missing
Preprints.org byte mapping and publication-status evidence
([[knowledge/crouzeix_conjecture/09_status_and_critical_assessment#evidence-that-would-raise-confidence|critical assessment]]).

For a senior ML systems reader, this is the most important lesson in the
artifact. The point is not that Lean magically turns a theorem into a press
release. The point is that a proof can be made operational: target statements,
source identities, proof attempts, builds, axiom audits, and prose claims can
all be separate, reviewable objects. That is the same discipline needed for
frontier-model evaluation and agentic research workflows. A benchmark score,
a trace, a proof script, and a narrative are different artifacts; collapsing
them is how error gets amplified.

## Why this proof belongs in Harp {#why-this-proof-belongs-in-harp}

Crouzeix is not an ML theorem, but it is an unusually good Harp test case. It
has a compact public statement, a deep hidden mechanism, multiple independent
candidate routes, source-provenance ambiguity, and a formalization boundary
that can be checked mechanically. It forces the repository to handle exactly
the work pattern Harp is meant to support:

- preserve source identity instead of trusting a summary;
- turn a mathematical claim into a formal target;
- run small proof slices with receipts;
- keep theorem status, source status, and publication status distinct;
- use generated reader artifacts only as derived views; and
- make the next agent able to resume from the evidence instead of from memory.

The proof mechanism is also a useful analogy for agentic engineering itself.
Jin does not win by making the unknown correction friendly. He wins by
carrying enough structure that the correction can be canceled at the right
interface. Good proof infrastructure works the same way. It does not ask the
reader to trust the whole transcript. It carries the right invariants until
the remaining claim is small enough to check.

Back to the [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Crouzeix conjecture index]].
