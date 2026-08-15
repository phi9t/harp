---
id: mathematical-foundations-orthogonality-spectra-and-decompositions
title: Orthogonality, spectra, positive-definite matrices, and decompositions
type: learning-module
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [linear-algebra, orthogonality, spectra, decompositions]
confidence: high
---

# Orthogonality, spectra, positive-definite matrices, and decompositions

Back to the [packet index](mathematical_foundations_index.md). Orthogonality
separates directions cleanly. Spectral and matrix factorizations make that
separation computationally visible in covariance, least squares, and PCA.

$$
x = \operatorname{proj}_S x + \bigl(x-\operatorname{proj}_S x\bigr),
\qquad x-\operatorname{proj}_S x \perp S.
$$

## MF-02-01 — Original problem

Compute the projection of $x=(3,1)$ onto $u=(1,1)$.

### Worked solution {#mf-02-01-solution}

$\operatorname{proj}_u x=(x^Tu/u^Tu)u=(4/2)(1,1)=(2,2)$. The residual
$(1,-1)$ is orthogonal to $u$.

## MF-02-02 — Original problem

Find the eigenvalues of $A=\begin{bmatrix}4&1\\1&4\end{bmatrix}$.

### Worked solution {#mf-02-02-solution}

$\det(A-\lambda I)=(4-\lambda)^2-1$, whose roots are $3$ and $5$.
The symmetric matrix therefore has real eigenvalues and orthogonal eigenvectors.

## MF-02-03 — Original problem

Is $Q=\frac1{5}\begin{bmatrix}3&4\\-4&3\end{bmatrix}$ orthogonal?

### Worked solution {#mf-02-03-solution}

Its columns have norm one and dot product $(12-12)/25=0$. Therefore
$Q^TQ=I$, so it preserves Euclidean lengths and angles.

## MF-02-04 — Original problem

Show that $A=\begin{bmatrix}2&1\\1&2\end{bmatrix}$ is positive definite.

### Worked solution {#mf-02-04-solution}

For $(x,y)\ne0$, $x^TAx=2x^2+2xy+2y^2=(x+y)^2+x^2+y^2>0$.
Equivalently its leading principal minors are $2$ and $3$, both positive.

## MF-02-05 — Original problem

For $A=\begin{bmatrix}3&0\\0&1\end{bmatrix}$, state the direction of
maximum stretch of $Ax$ over unit vectors.

### Worked solution {#mf-02-05-solution}

The first coordinate direction $(1,0)$ is stretched by $3$, while the
second is stretched by $1$. Thus the maximum is attained in the first
direction, the right singular vector for singular value $3$.

## MF-02-06 — Original problem

Let $C=\begin{bmatrix}1&2\\2&4\end{bmatrix}$. Is it a valid covariance
matrix? Is it invertible?

### Worked solution {#mf-02-06-solution}

$C=(1,2)^T(1,2)$, hence it is symmetric positive semidefinite and can be a
covariance matrix. Its determinant is zero, so it is not invertible; the two
coordinates have perfect linear dependence.

## MF-02-07 — Original problem

If $X=U\Sigma V^T$ has singular values $6,2,0$, what is its rank and
what error remains after retaining only the largest singular value?

### Worked solution {#mf-02-07-solution}

The rank is two. The best rank-one approximation leaves squared Frobenius error
$2^2+0^2=4$, because discarded singular values measure the residual energy.

## MF-02-08 — Original problem

Why is the Cholesky factorization useful for a positive-definite covariance
matrix $K=LL^T$?

### Worked solution {#mf-02-08-solution}

It represents $K$ using a triangular factor, so solves use forward and back
substitution rather than an inverse. Also, if $z\sim\mathcal N(0,I)$, then
$Lz\sim\mathcal N(0,K)$, enabling Gaussian sampling.
