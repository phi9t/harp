---
id: mathematical-foundations-glossary
title: Mathematical foundations glossary
type: glossary
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [glossary, notation, mathematics, machine-learning]
confidence: high
---

# Glossary

Back to the [packet index](mathematical_foundations_index.md).

- **Basis:** linearly independent vectors that span a vector space. Coordinates
  depend on a basis; the underlying vector does not.
- **Covariance:** $\operatorname{Cov}(X,Y)=\mathbb E[(X-\mathbb EX)(Y-\mathbb EY)]$.
  A covariance matrix records pairwise covariance.
- **Eigenpair:** $(\lambda,v)$ with $Av=\lambda v$ and $v\ne0$.
- **Gradient:** vector of first partial derivatives. Its negative is the local
  steepest-descent direction under the Euclidean norm.
- **Hessian:** matrix of second partial derivatives. Positive definiteness at a
  stationary point is a local-minimum certificate.
- **Inner product:** a bilinear (or sesquilinear) positive pairing $\langle
  x,y\rangle$, inducing $\|x\|=\sqrt{\langle x,x\rangle}$.
- **Likelihood:** viewed as a function of parameters, $p(D\mid\theta)$ for
  fixed observed data $D$.
- **MAP estimate:** parameter maximizing $p(\theta\mid D)$, equivalently a
  likelihood times a prior when the evidence is constant in $\theta$.
- **Positive definite:** symmetric $A$ with $x^TAx>0$ for all nonzero $x$.
- **Regularization:** a penalty or constraint that selects among fits, often
  reducing variance or encoding a preference for small parameters.
- **Residual:** prediction error $r=y-\hat y$. Least squares minimizes
  $\|r\|_2^2$.
- **Singular value decomposition:** $A=U\Sigma V^T$, exposing orthogonal
  input and output directions and nonnegative gains.
- **Trace:** $\operatorname{tr}(A)=\sum_i A_{ii}$; it is invariant under
  cyclic products where dimensions agree.
