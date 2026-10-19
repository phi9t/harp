---
id: mathematical-foundations-linear-models-and-regularization
title: Linear regression, classification, regularization, and probabilistic models
type: learning-module
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [regression, classification, regularization, probabilistic-models]
confidence: high
---

# Linear regression, classification, regularization, and probabilistic models

Back to the [packet index](mathematical_foundations_index.md). A linear model
is linear in its parameters, even when its inputs are transformed. Loss and
regularization define what “fit” means and which solution is preferred.

$$
X^T X w = X^T y.
$$

## MF-05-01 — Original problem

For data $(x,y)=(1,2),(2,4)$ and model $\hat y=wx$, find the least-squares
coefficient $w$.

### Worked solution {#mf-05-01-solution}

Minimize $(2-w)^2+(4-2w)^2$. Setting its derivative to zero gives
$10w-20=0$, so $w=2$ and both residuals vanish.

## MF-05-02 — Original problem

Let $X$ have full column rank. State the normal equations for least squares
$\min_w\|Xw-y\|_2^2$.

### Worked solution {#mf-05-02-solution}

Differentiation gives $2X^T(Xw-y)=0$, hence $X^TXw=X^Ty$. Full column
rank makes $X^TX$ invertible, so $w=(X^TX)^{-1}X^Ty$.

## MF-05-03 — Original problem

For a binary label, logistic regression produces score $s=w^Tx$. Write the
predicted positive probability and state why it lies in $(0,1)$.

### Worked solution {#mf-05-03-solution}

$p(y=1\mid x)=\sigma(s)=1/(1+e^{-s})$. The denominator exceeds one and is
finite, so the result is strictly between zero and one.

## MF-05-04 — Original problem

One example has label $y=1$ and predicted probability $0.8$. What is its
binary cross-entropy loss?

### Worked solution {#mf-05-04-solution}

The loss is $-\log(0.8)$, approximately $0.223$ in natural units. Only
the probability assigned to the observed class contributes for this example.

## MF-05-05 — Original problem

Compare the effect of $\lambda\|w\|_2^2$ and $\lambda\|w\|_1$ penalties
on a linear predictor.

### Worked solution {#mf-05-05-solution}

Both discourage large weights. The squared $\ell_2$ penalty usually shrinks
weights smoothly; the $\ell_1$ penalty has corners at zero and can set some
coefficients exactly to zero, producing sparse models.

## MF-05-06 — Original problem

Why is a bias/intercept column of ones commonly appended to a design matrix?

### Worked solution {#mf-05-06-solution}

It lets $Xw$ include a constant offset using the same matrix notation. With
no intercept, a linear regression surface is forced through the origin in its
feature coordinates.

## MF-05-07 — Original problem

What predictive distribution results from assuming $y=w^Tx+\epsilon$ with
$\epsilon\sim\mathcal N(0,\sigma^2)$?

### Worked solution {#mf-05-07-solution}

Conditional on $x,w$, $y\sim\mathcal N(w^Tx,\sigma^2)$. Maximizing this
Gaussian likelihood with fixed variance is equivalent to minimizing squared
residuals, up to constants and a positive scale.

## MF-05-08 — Original problem

A validation curve falls then rises as polynomial degree increases. What does
the rising part suggest, and what action is reasonable?

### Worked solution {#mf-05-08-solution}

The model is likely fitting training-specific variation rather than transferable
signal: variance has overtaken the bias benefit. Choose a lower validated
degree, add regularization, or collect more representative data.
