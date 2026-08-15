---
id: mathematical-foundations-linear-spaces-and-maps
title: Linear spaces, maps, bases, and matrices
type: learning-module
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [linear-algebra, vector-spaces, matrices, machine-learning]
confidence: high
---

# Linear spaces, maps, bases, and matrices

Back to the [packet index](mathematical_foundations_index.md). A feature vector
is useful because linear combinations preserve its representation. A matrix is
best understood as a linear map whose columns say where basis vectors go.

$$
A e_j = \text{column } j \text{ of } A.
$$

## MF-01-01 — Original problem

Let $u=(1,2)$, $v=(3,-1)$, and $w=(7,3)$. Express $w$ as
$au+bv$, if possible.

### Worked solution {#mf-01-01-solution}

Solve $a+3b=7$ and $2a-b=3$. The second equation gives $b=2a-3$;
substitution yields $7a-9=7$, so $a=16/7$ and $b=11/7$.

## MF-01-02 — Original problem

For $A=\begin{bmatrix}1&2\\0&-1\end{bmatrix}$, compute $A(3,4)^T$
and explain the result using columns.

### Worked solution {#mf-01-02-solution}

$A(3,4)^T=(11,-4)^T$. Equivalently it is three copies of the first
column plus four copies of the second: $3(1,0)^T+4(2,-1)^T$.

## MF-01-03 — Original problem

Does $S=\{(x,y,z):x+y+z=0\}$ form a subspace of $\mathbb R^3$? Give a
basis.

### Worked solution {#mf-01-03-solution}

Yes: it is the kernel of the linear map $(x,y,z)\mapsto x+y+z$. Write
$(x,y,z)=x(1,0,-1)+y(0,1,-1)$, so the displayed two vectors form a basis.

## MF-01-04 — Original problem

The columns of $X\in\mathbb R^{4\times3}$ are features. What does the
condition $Xc=0$ with $c\ne0$ say about those features?

### Worked solution {#mf-01-04-solution}

It says a nontrivial weighted sum of feature columns is zero. Thus the columns
are linearly dependent and at least one feature is redundant as a linear
combination of the others.

## MF-01-05 — Original problem

Let $T(x,y)=(x+y,2x-y)$. Find its matrix in the standard basis and compute
$T^{-1}$, if it exists.

### Worked solution {#mf-01-05-solution}

The columns are $T(1,0)=(1,2)$ and $T(0,1)=(1,-1)$, hence
$A=\begin{bmatrix}1&1\\2&-1\end{bmatrix}$. Its determinant is $-3$, so
$A^{-1}=\frac1{-3}\begin{bmatrix}-1&-1\\-2&1\end{bmatrix}$.

## MF-01-06 — Original problem

If a dataset matrix has rank $r$, how many independent directions can its
rows occupy? What does rank $0$ mean?

### Worked solution {#mf-01-06-solution}

Its row space has exactly $r$ independent directions. Rank $0$ means all
entries are zero, so every observation vector is the zero vector.

## MF-01-07 — Original problem

Let $B=\{(1,1),(1,-1)\}$. Find the $B$-coordinates of $(5,1)$.

### Worked solution {#mf-01-07-solution}

Set $a(1,1)+b(1,-1)=(5,1)$. Then $a+b=5$, $a-b=1$, giving
$(a,b)=(3,2)$. Thus $[(5,1)]_B=(3,2)^T$.

## MF-01-08 — Original problem

Suppose $P$ is a square matrix satisfying $P^2=P$. Show that applying
$P$ twice to an input changes nothing after the first application.

### Worked solution {#mf-01-08-solution}

For any $x$, $P(Px)=P^2x=Px$. Such a map is a projection onto its image;
ML preprocessing uses this pattern when it removes a component once and keeps
the remaining component unchanged.
