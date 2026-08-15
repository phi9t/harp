---
id: mathematical-foundations-probability-and-gaussian-models
title: Probability, expectation, covariance, and Gaussian models
type: learning-module
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [probability, expectation, covariance, gaussian-models]
confidence: high
---

# Probability, expectation, covariance, and Gaussian models

Back to the [packet index](mathematical_foundations_index.md). Probability
models uncertainty; expectation summarizes a distribution; covariance records
which deviations move together. Gaussian models package all three efficiently.

$$
\operatorname{Cov}(X,Y)=\mathbb E[(X-\mathbb EX)(Y-\mathbb EY)].
$$

## MF-03-01 — Original problem

A fair six-sided die is rolled. Compute $\mathbb P(X\ge5)$ and $\mathbb EX$.

### Worked solution {#mf-03-01-solution}

Two faces satisfy $X\ge5$, so the probability is $2/6=1/3$. Symmetry or
averaging $(1+\cdots+6)/6$ gives $\mathbb EX=3.5$.

## MF-03-02 — Original problem

For a Bernoulli variable with $\mathbb P(X=1)=p$, derive its variance.

### Worked solution {#mf-03-02-solution}

Because $X^2=X$, $\mathbb EX=p$ and $\mathbb EX^2=p$. Therefore
$\operatorname{Var}(X)=p-p^2=p(1-p)$.

## MF-03-03 — Original problem

If $Y=3X-2$, express $\mathbb EY$ and $\operatorname{Var}(Y)$ in
terms of the mean $\mu$ and variance $\sigma^2$ of $X$.

### Worked solution {#mf-03-03-solution}

Linearity gives $\mathbb EY=3\mu-2$. Centering yields
$Y-\mathbb EY=3(X-\mu)$, so $\operatorname{Var}(Y)=9\sigma^2$.

## MF-03-04 — Original problem

Let $X$ be equally likely to be $-1$ or $1$, and let $Y=X$. Find
$\operatorname{Cov}(X,Y)$.

### Worked solution {#mf-03-04-solution}

Both means are zero and $XY=X^2=1$. Hence the covariance is
$\mathbb E[XY]-0=1$. The variables are perfectly positively correlated.

## MF-03-05 — Original problem

Two independent measurements each have mean $10$ and variance $4$. What
are the mean and variance of their average?

### Worked solution {#mf-03-05-solution}

The average has mean $10$. Independence makes its variance
$(4+4)/2^2=2$, illustrating why averaging reduces uncorrelated noise.

## MF-03-06 — Original problem

For $Z\sim\mathcal N(0,1)$, what is the density ratio $p(1)/p(0)$?

### Worked solution {#mf-03-06-solution}

The normalizing constant cancels, leaving
$\exp(-1^2/2)/\exp(0)=e^{-1/2}$. Ratios often avoid unnecessary density
normalizers.

## MF-03-07 — Original problem

Let $x\sim\mathcal N(\mu,\Sigma)$ and $y=Ax+b$. Give the mean and
covariance of $y$.

### Worked solution {#mf-03-07-solution}

$\mathbb Ey=A\mu+b$. Centering gives $y-\mathbb Ey=A(x-\mu)$, hence
$\operatorname{Cov}(y)=A\Sigma A^T$. The result remains Gaussian.

## MF-03-08 — Original problem

Why does a covariance matrix with a tiny eigenvalue warn of a numerical issue
when evaluating a Gaussian density?

### Worked solution {#mf-03-08-solution}

The inverse covariance weights that eigen-direction by the reciprocal tiny
eigenvalue, magnifying noise. Its determinant is also tiny, so the density is
concentrated near a lower-dimensional direction; regularization may be needed.
