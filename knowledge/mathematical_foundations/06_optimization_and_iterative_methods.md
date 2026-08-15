---
id: mathematical-foundations-optimization-and-iterative-methods
title: Gradients, Hessians, optimization, and iterative linear-system methods
type: learning-module
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [optimization, gradients, hessians, iterative-methods]
confidence: high
---

# Gradients, Hessians, optimization, and iterative linear-system methods

Back to the [packet index](mathematical_foundations_index.md). Optimization
connects an objective to an update rule. Gradients provide first-order local
information, Hessians describe curvature, and iterative solvers exploit matrix
structure without explicitly computing inverses.

$$
w_{t+1}=w_t-\eta\nabla f(w_t).
$$

## MF-06-01 — Original problem

For $f(x,y)=x^2+3xy$, compute $\nabla f$ at $(1,2)$.

### Worked solution {#mf-06-01-solution}

$\nabla f=(2x+3y,3x)$, so at $(1,2)$ it is $(8,3)$. A small negative
multiple of this vector is the steepest local decrease direction.

## MF-06-02 — Original problem

Perform one gradient-descent step with learning rate $0.1$ on
$f(w)=(w-4)^2$ from $w=0$.

### Worked solution {#mf-06-02-solution}

The derivative is $2(w-4)$, equal to $-8$ at zero. The update is
$w\leftarrow0-0.1(-8)=0.8$, which moves toward the minimizer $4$.

## MF-06-03 — Original problem

Find the Hessian of $f(x,y)=x^2+xy+2y^2$, and say whether it is positive
definite.

### Worked solution {#mf-06-03-solution}

$H=\begin{bmatrix}2&1\\1&4\end{bmatrix}$. Its leading principal minors
are $2$ and $7$, both positive, so it is positive definite and the
quadratic has a unique global minimizer.

## MF-06-04 — Original problem

Why can a learning rate that is too large make gradient descent diverge on a
quadratic bowl?

### Worked solution {#mf-06-04-solution}

The update can overshoot the minimum and land farther away on the opposite
side. Repeated overshoots grow when the update multiplier along a curvature
direction has magnitude greater than one.

## MF-06-05 — Original problem

For $f(w)=\tfrac12w^TAw-b^Tw$ with symmetric $A$, write its gradient.

### Worked solution {#mf-06-05-solution}

$\nabla f(w)=Aw-b$. Setting it to zero gives $Aw=b$, so minimizing a
positive-definite quadratic is equivalent to solving its associated linear
system.

## MF-06-06 — Original problem

What residual should be monitored while iteratively solving $Aw=b$, and why?

### Worked solution {#mf-06-06-solution}

Monitor $r=b-Aw$. It is zero exactly when the current iterate solves the
system, and its norm measures violation of the equation without requiring the
unknown exact solution.

## MF-06-07 — Original problem

Why does conjugate gradient target symmetric positive-definite systems rather
than arbitrary matrices?

### Worked solution {#mf-06-07-solution}

Positive definiteness supplies a convex quadratic objective and an
$A$-weighted inner product. These structures make its mutually conjugate
search directions well behaved and ensure a unique solution.

## MF-06-08 — Original problem

State one reason minibatch gradients are used instead of full-data gradients
in large ML training.

### Worked solution {#mf-06-08-solution}

A minibatch gives a much cheaper estimate of the full gradient, allowing more
frequent updates. It introduces sampling noise, but that trade can be favorable
when a full pass through the data is expensive.
