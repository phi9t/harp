---
id: crouzeix-textbook-notation-and-glossary
title: Notation and glossary
type: glossary
status: active
created: 2026-08-23
updated: 2026-09-04
tags: [crouzeix-textbook, notation, glossary, mathematics]
confidence: high
canonical: notation_and_glossary.md
---

# Notation and glossary

Back to the [[knowledge/crouzeix_textbook/crouzeix_textbook_index|book index]].

## Logical vocabulary

- **Set:** a collection whose members have an unambiguous membership test in
  the surrounding mathematical theory.
- **Function:** an assignment sending every element of a domain to exactly one
  element of a codomain. The codomain is part of the declared function.
- **Definition:** a declaration fixing the meaning of a term or symbol.
- **Proposition:** a statement that is either true or false in the theory.
- **Theorem:** a proposition accompanied by a proof.
- **Proof:** a finite argument reducing a claim to accepted definitions,
  hypotheses, and earlier results. In Lean, a proof term inhabits the theorem's
  proposition type.
- **Counterexample:** an object satisfying a claim's hypotheses but not its
  conclusion. One counterexample refutes a universal statement.

## Linear vocabulary

- **Scalar field or ring $\mathbb K$:** the numbers used to scale vectors. Most
  analytic chapters use $\mathbb R$ or $\mathbb C$.
- **Vector space:** a set with vector addition and scalar multiplication
  satisfying the vector-space laws.
- **Subspace:** a subset closed under zero, addition, and scalar multiplication.
- **Linear transformation:** a function $T:V\to W$ satisfying
  $T(x+y)=T(x)+T(y)$ and $T(ax)=aT(x)$.
- **Basis:** a linearly independent spanning family. A basis gives unique
  coordinates but is not part of the underlying vector unless declared.
- **Coordinate vector $[x]_B$:** the scalar tuple representing $x$ in basis
  $B$.
- **Matrix $[T]_{C\leftarrow B}$:** the array representing $T$ from domain
  basis $B$ to codomain basis $C$.
- **Similarity:** the relation $A'=SAS^{-1}$. Similar matrices represent the
  same endomorphism in different bases.
- **Invariant:** a quantity unchanged under the equivalence relation currently
  under study. Characteristic polynomial is a similarity invariant; Euclidean
  operator norm is not invariant under arbitrary similarity.

## Analytic vocabulary introduced early

- **Norm $\lVert x\rVert$:** a nonnegative size satisfying definiteness,
  homogeneity, and the triangle inequality.
- **Operator norm:** $\lVert T\rVert=\sup_{x\ne0}\lVert Tx\rVert/\lVert x\rVert$.
- **Spectrum:** for a finite matrix, the set of roots of its characteristic
  polynomial, counted without multiplicity when treated as a set.
- **Nonnormal matrix:** over $\mathbb C$, a matrix $A$ for which
  $A^*A\ne AA^*$. Nonnormality permits geometric behavior not determined by
  eigenvalues alone.

## Finite-horizon proof vocabulary

The [[knowledge/crouzeix_textbook/part_06_constant_two_routes/36_harp_finite_horizon_proof|Harp proof chapter]]
uses the following conventions. These are notation guides, not additional theorem claims.

- **Adjoint and inner product:** $T^*$ denotes the Hilbert-space adjoint;
  $\langle x,y\rangle$ is conjugate-linear in $x$ and linear in $y$.
- **Positive cubature:** finitely many nodes with strictly positive scalar
  weights reproduce a specified finite family of integrals exactly. This is
  an existence statement, not a floating-point error estimate.
- **Matrix mass:** $D_j$ are positive semidefinite matrix weights satisfying
  $\sum_j D_j=2I$. They are distinct from the scalar cubature weights.
- **Finite-horizon dilation:** for each natural number $N$, an isometric
  embedding $V_N$ and contraction $Q_N$ reproduce the required moments through
  order $N+1$. Their ambient space may depend on $N$.
- **Fixed core:** $T$, the commuting companion operators $E_k$, and their
  common bound remain fixed as $N$ varies. In particular, the scalar sequence
  $m_k=\operatorname{Re}\langle x,E_kT^kx\rangle$ is independent of the
  chosen horizon; the displacement $b_N$ need not be.

## Lean notation

- `V →ₗ[𝕜] W` is the type of $\mathbb K$-linear maps.
- `Matrix n n 𝕜` is a matrix whose row and column indices have type `n`.
- `A *ᵥ x` is matrix-vector multiplication.
- `Pi.single j 1` is the coordinate vector with value one at `j` and zero
  elsewhere.
- A declaration after `#check` is elaborated by Lean; `#check` does not prove a
  new theorem, but it ensures that the named declaration exists with a type.
