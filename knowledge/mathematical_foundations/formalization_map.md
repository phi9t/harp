---
id: math-foundations-formalization-map
title: Mathematical foundations formalization map
type: formalization-map
status: active
created: 2026-08-15
updated: 2026-09-08
tags: [mathematics, formalization, lean, machine-learning]
confidence: high
---

# Mathematical foundations formalization map

Back to the [packet index](mathematical_foundations_index.md). This is the
reader-facing boundary for the accompanying Lean project: a named theorem is a
compiled statement over its stated domain, while a prose-only row makes no
Lean-proof claim. The finite-coordinate formalization uses expressions such as
$x \in \mathbb R^n$; it is not a proof of an empirical modelling conclusion.

## Status key

- **Direct theorem** — the original problem statement is a compiled named
  theorem.
- **Corollary/application** — the worked solution uses a compiled theorem, but
  still needs the displayed instance or calculation.
- **Prose-only** — the row states an exact boundary and has no Lean identifier.

## Compiled declaration inventory

The following are the public compiled theorems in the six supported namespaces.
They are listed here so the map does not imply that unlisted Lean results exist.

| Module | Compiled public theorems |
| --- | --- |
| [Module 1: linear spaces and maps](01_linear_spaces_and_maps.md) | `MathematicalFoundations.Linear.linear_map_zero`; `MathematicalFoundations.Linear.linear_map_add`; `MathematicalFoundations.Linear.linear_map_smul`; `MathematicalFoundations.Linear.linear_kernel_zero`; `MathematicalFoundations.Linear.linear_kernel_add`; `MathematicalFoundations.Linear.linear_kernel_smul`; `MathematicalFoundations.Linear.linear_finite_coordinate_reconstruction` |
| [Module 2: orthogonality](02_orthogonality_spectra_and_decompositions.md) | `MathematicalFoundations.Orthogonality.line_projection_residual_decomposition`; `MathematicalFoundations.Orthogonality.line_projection_residual_inner_eq_zero`; `MathematicalFoundations.Orthogonality.symmetric_positive_definite_quadratic_positive` |
| [Module 3: probability](03_probability_and_gaussian_models.md) | `MathematicalFoundations.Probability.expectation_add`; `MathematicalFoundations.Probability.expectation_smul`; `MathematicalFoundations.Probability.expectation_sub`; `MathematicalFoundations.Probability.expectation_const`; `MathematicalFoundations.Probability.covariance_expand` |
| [Module 4: Bayes and information](04_bayesian_inference_and_information.md) | `MathematicalFoundations.BayesInformation.eventProbability_and_comm`; `MathematicalFoundations.BayesInformation.conditional_mul_evidence`; `MathematicalFoundations.BayesInformation.bayes_rule` |
| [Module 5: linear models](05_linear_models_and_regularization.md) | `MathematicalFoundations.LinearModels.normal_equation_residual_identity`; `MathematicalFoundations.LinearModels.ridge_objective_zero_coefficients` |
| [Module 6: optimization](06_optimization_and_iterative_methods.md) | `MathematicalFoundations.Optimization.scalar_gradient_descent_error_recurrence`; `MathematicalFoundations.Optimization.scalar_stationary_iteration_succ`; `MathematicalFoundations.Optimization.scalar_stationary_iteration_tendsto_fixed_point` |

## Module 1 — linear spaces and maps

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-01-01 | Prose-only | — | Exact limitation: the numerical coordinate solve for these three vectors has no compiled declaration. |
| MF-01-02 | Corollary/application | Lean: `MathematicalFoundations.Linear.linear_map_add`; `MathematicalFoundations.Linear.linear_map_smul` | The column-combination explanation is an application; the displayed matrix-vector arithmetic is not separately formalized. |
| MF-01-03 | Corollary/application | Lean: `MathematicalFoundations.Linear.linear_kernel_zero`; `MathematicalFoundations.Linear.linear_kernel_add`; `MathematicalFoundations.Linear.linear_kernel_smul` | Kernel closure supports the subspace portion; the particular two-vector basis is not separately formalized. |
| MF-01-04 | Prose-only | — | Exact limitation: the matrix-column dependence interpretation is not represented by a compiled theorem. |
| MF-01-05 | Prose-only | — | Exact limitation: this concrete inverse calculation is not represented by a compiled theorem. |
| MF-01-06 | Prose-only | — | Exact limitation: rank and row-space dimension are outside the companion's theorem boundary. |
| MF-01-07 | Corollary/application | Lean: `MathematicalFoundations.Linear.linear_finite_coordinate_reconstruction` | Coordinatewise equality supports reconstruction after the displayed numerical solve. |
| MF-01-08 | Prose-only | — | Exact limitation: no compiled idempotent-matrix theorem is included. |

## Module 2 — orthogonality, spectra, and decompositions

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-02-01 | Corollary/application | Lean: `MathematicalFoundations.Orthogonality.line_projection_residual_decomposition`; `MathematicalFoundations.Orthogonality.line_projection_residual_inner_eq_zero` | The residual decomposition and orthogonality are compiled for finite real vectors; the displayed numeric projection still needs arithmetic. |
| MF-02-02 | Prose-only | — | Exact limitation: no compiled eigenvalue calculation or spectral theorem is included. |
| MF-02-03 | Prose-only | — | Exact limitation: no compiled orthogonal-matrix verification for this concrete matrix is included. |
| MF-02-04 | Prose-only | — | Exact limitation: the companion does not prove positive definiteness for this concrete matrix from its entries. |
| MF-02-05 | Prose-only | — | Exact limitation: singular-value maximization is outside the companion's theorem boundary. |
| MF-02-06 | Prose-only | — | Exact limitation: covariance-matrix validity and invertibility for this matrix are not formalized. |
| MF-02-07 | Prose-only | — | Exact limitation: rank-one approximation and Frobenius-error claims are not formalized. |
| MF-02-08 | Prose-only | — | Exact limitation: Cholesky factorization and Gaussian sampling are not formalized. |

## Module 3 — probability and Gaussian models

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-03-01 | Prose-only | — | Exact limitation: the fair-die evaluation is not encoded as a particular finite probability mass function. |
| MF-03-02 | Prose-only | — | Exact limitation: the Bernoulli variance formula is not a compiled theorem. |
| MF-03-03 | Corollary/application | Lean: `MathematicalFoundations.Probability.expectation_add`; `MathematicalFoundations.Probability.expectation_smul`; `MathematicalFoundations.Probability.expectation_sub`; `MathematicalFoundations.Probability.expectation_const` | The finite-PMF expectation transformation is supported; the variance-scaling portion is not a compiled theorem. |
| MF-03-04 | Corollary/application | Lean: `MathematicalFoundations.Probability.covariance_expand` | The covariance expansion supports the two-point calculation after choosing its finite mass function. |
| MF-03-05 | Prose-only | — | Exact limitation: independence and the variance of an average are not formalized. |
| MF-03-06 | Prose-only | — | Exact limitation: Gaussian densities and their ratio are outside the finite-PMF boundary. |
| MF-03-07 | Prose-only | — | Exact limitation: affine transformations of Gaussian distributions are not formalized. |
| MF-03-08 | Prose-only | — | Exact limitation: covariance conditioning and Gaussian-density numerics are not formalized. |

## Module 4 — Bayesian inference and information

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-04-01 | Corollary/application | Lean: `MathematicalFoundations.BayesInformation.eventProbability_and_comm`; `MathematicalFoundations.BayesInformation.conditional_mul_evidence`; `MathematicalFoundations.BayesInformation.bayes_rule` | Bayes' identity is compiled for finite probability mass functions with nonzero prior and nonzero evidence; the displayed decimal calculation remains an application. |
| MF-04-02 | Prose-only | — | Exact limitation: Beta-Bernoulli conjugacy is not formalized. |
| MF-04-03 | Prose-only | — | Exact limitation: MAP and maximum-likelihood optimization are not formalized. |
| MF-04-04 | Prose-only | — | Exact limitation: entropy in bits is not formalized. |
| MF-04-05 | Prose-only | — | Exact limitation: mutual information and its independence identity are not formalized. |
| MF-04-06 | Prose-only | — | Exact limitation: this model-selection judgment is not a compiled mathematical claim. |
| MF-04-07 | Prose-only | — | Exact limitation: posterior predictive integration is not formalized. |
| MF-04-08 | Prose-only | — | Exact limitation: independent-observation likelihood factorization is not formalized. |

## Module 5 — linear models and regularization

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-05-01 | Corollary/application | Lean: `MathematicalFoundations.LinearModels.normal_equation_residual_identity` | The zero-normal-residual algebra is compiled; the least-squares minimizer calculation is still an application. |
| MF-05-02 | Prose-only | — | Exact limitation: full-rank invertibility and the minimizer characterization are not compiled. |
| MF-05-03 | Prose-only | — | Exact limitation: logistic probabilities and their bounds are not formalized. |
| MF-05-04 | Prose-only | — | Exact limitation: binary cross-entropy evaluation is not formalized. |
| MF-05-05 | Prose-only | — | Exact limitation: the comparative sparsity behavior of $\ell_1$ and $\ell_2$ penalties is not formalized. |
| MF-05-06 | Prose-only | — | Exact limitation: the intercept-column modelling interpretation is not a compiled theorem. |
| MF-05-07 | Prose-only | — | Exact limitation: the Gaussian-noise predictive distribution and likelihood equivalence are not formalized. |
| MF-05-08 | Prose-only | — | Exact limitation: validation-curve interpretation is empirical modelling guidance, not a compiled theorem. |

## Module 6 — optimization and iterative methods

| Original problem | Status | Lean declaration | Scope or limitation |
| --- | --- | --- | --- |
| MF-06-01 | Prose-only | — | Exact limitation: the displayed two-variable gradient calculation is not formalized. |
| MF-06-02 | Corollary/application | Lean: `MathematicalFoundations.Optimization.scalar_gradient_descent_error_recurrence` | The scalar quadratic error recurrence supports the one-step calculation after substituting its parameters. |
| MF-06-03 | Prose-only | — | Exact limitation: the concrete Hessian and positive-definiteness calculation are not formalized. |
| MF-06-04 | Prose-only | — | Exact limitation: the scalar stationary-iteration results prove convergence under an explicit contraction, not divergence from an excessively large gradient-descent learning rate. |
| MF-06-05 | Prose-only | — | Exact limitation: the vector quadratic gradient and linear-system equivalence are not formalized. |
| MF-06-06 | Prose-only | — | Exact limitation: residual monitoring for a matrix iteration is not formalized. |
| MF-06-07 | Prose-only | — | Exact limitation: conjugate-gradient correctness for positive-definite systems is not formalized. |
| MF-06-08 | Prose-only | — | Exact limitation: the minibatch tradeoff is an empirical computational judgment, not a compiled theorem. |

## Cauchy-Schwarz educational proof laboratory: design

Design recorded 2026-09-08. The user approved four from-scratch finite-real
proof routes: quadratic nonnegativity, Lagrange identity, normalization, and
induction. Additional routes below are researched curriculum proposals, not
claims of implementation. The existing `TextbookBench.cauchySchwarzReference`
is a library-reuse control only. This section does not change the compiled
coverage status of any row above.

### Purpose and success criteria

Teach a learner to discover the useful intermediate statement, translate it
into Lean, explain each hypothesis, and recognize the same mathematical idea
in another representation. A kernel-accepted proof is necessary, but does not
establish understanding, originality, pedagogical quality, or safe execution.

Use three separate records: mathematical validity, compliance with the
exercise's permitted dependencies, and human-reviewed explanation quality.
Never collapse them into a single proof score. The first delivery succeeds
only when all four core routes prove the exact target, pass their own controls,
pass the dependency policy, and have a readable sequence of intermediate
lemmas. A four-route module compiling after importing the reference proof is
not sufficient evidence.

### Fixed target and hypotheses

For arbitrary natural `n`, and `x y : Fin n → ℝ`, define

$$A=\sum_i x_i^2,\qquad B=\sum_i y_i^2,\qquad C=\sum_i x_i y_i.$$

Every core route must prove $C^2\le AB$, with no assumption that $n>0$, that
coordinates are positive, or that either vector is nonzero. Keep the public
target byte-stable where practical; bind acceptance to its elaborated type
in a trusted environment, not merely its printed name or source hash.

Prove elementary helpers for $A,B\ge0$ and
$B=0\Rightarrow\forall i,\ y_i=0$. The latter follows by bounding each
nonnegative square by the sum, then using $y_i^2=0$. These helpers can be shared.
Sharing a helper that already entails the target is prohibited for independent
core routes. Document all shared helpers and review this distinction manually.

### Four core constructions

**CS-Q: quadratic nonnegativity.** Expand the finite sum to prove, for every
real $t$, $0\le A-2tC+t^2B$. Handle $B=0$ with the zero-square helper. For
$B>0$, substitute $t=C/B$, clear the positive denominator, and conclude
$0\le AB-C^2$. Do not invoke a packaged discriminant theorem that embeds this
entire argument. A separately derived scalar quadratic lemma is acceptable
if its proof is part of the exercise. Learning goal: choose a parameter that
eliminates the mixed term. Record the division side condition before clearing
denominators.

**CS-L: Lagrange identity.** Derive the identity

$$\sum_i\sum_j (x_i y_j-x_j y_i)^2=2(AB-C^2)$$

by expanding each square, distributing finite sums, factoring products, and
renaming indices. Nonnegativity of each summand gives the target. This route
requires no division or square roots and naturally includes the empty set.
Do not import an existing Lagrange or Cauchy-Binet identity as the proof step.
Learning goal: reverse-engineer an error term that vanishes at equality.
Keep the double-sum identity named so the reader can inspect it independently.

**CS-N: normalization.** After disposing of $A=0$ or $B=0$, set
$u_i=x_i/\sqrt A$ and $v_i=y_i/\sqrt B$. Prove that both sums of squares
equal one. Expanding both $\sum_i(u_i-v_i)^2\ge0$ and
$\sum_i(u_i+v_i)^2\ge0$ gives $-1\le\sum_i u_i v_i\le1$.
Square only after establishing this two-sided bound, then rescale.
Learning goal: exploit homogeneity and understand why an upper bound alone
cannot be squared. Require only lower-level square-root identities and order
facts, never a norm or normalized-inner-product bound.

**CS-I: induction.** Use finite-set insertion as the internal induction domain,
then specialize to `Finset.univ`. The empty set is the base case. For the
previous partial sums $A,B,C$ and a newly inserted pair $a,b$, derive

$$ (a^2B+b^2A)^2-(2abC)^2
= (a^2B-b^2A)^2+4a^2b^2(AB-C^2)\ge0.$$

Since $a^2B+b^2A\ge0$, obtain $2abC\le a^2B+b^2A$ by elementary ordered-field
reasoning, then expand
$(A+a^2)(B+b^2)-(C+ab)^2=(AB-C^2)+a^2B+b^2A-2abC$.
This yields the insertion step without invoking CS-Q, CS-L, or CS-N. Learning
goal: strengthen the induction hypothesis enough to control the mixed term.
The scalar square comparison must itself be derived or explicitly approved
as a low-level order lemma. This is our chosen formalization construction,
not a claim that Steele prints this exact induction argument.

### Additional proof presentations and extensions

The following classification is a mathematical design assessment. Different
presentations can have educational value without being independent proof
mechanisms. Exact source support and access limitations are recorded below.

| ID | Presentation and construction | Family and prerequisite burden | Educational disposition |
| --- | --- | --- | --- |
| CS-P | Orthogonal projection: set $r=x-(C/B)y$, expand $\sum r_i^2=A-C^2/B$ | Same completed-square mechanism as CS-Q; coordinate algebra suffices | Teach after CS-Q; bridge to Lax and least squares, not a fifth independent core proof |
| CS-G | Gram matrix: expand $(s,t)\begin{pmatrix}A&C\\C&B\end{pmatrix}(s,t)^T=\sum_i(sx_i+ty_i)^2$; derive nonnegative determinant for this two-by-two matrix | Quadratic family; matrix notation adds representation burden | Teach PSD versus positive definite; derive the determinant criterion rather than citing a theorem that uses CS |
| CS-V | Weighted variance: for $A>0$, restrict to $x_i\ne0$, put $p_i=x_i^2/A$, $z_i=y_i/x_i$; expand $\sum_i p_i(z_i-\sum_jp_jz_j)^2\ge0$ | Weighted square family; zero support and mass-one obligations are essential | Connect to Bishop; recover $C^2\le A\sum_{x_i\ne0}y_i^2\le AB$ |
| CS-J | Convexity of the square: derive finite weighted Jensen for $z\mapsto z^2$ by induction from the two-point square identity, then use CS-V weights | Shares variance algebra; a generic Jensen theorem can hide target-equivalent work | Extension exercise in abstraction and circularity, not a shortcut allowed in core routes |
| CS-Y | Scaled Young inequality: derive $2uv\le u^2+v^2$ from $(u-v)^2\ge0$, sum after scaling and optimize the scale | Normalization/quadratic family; handles signs and zero scale | A short alternate presentation, explicitly linked to CS-N |
| CS-O | Constrained maximization of the linear functional on $\sum x_i^2=1$ | Compactness and multiplier/stationary-point machinery may already depend on CS in norm infrastructure | Advanced optional study only after dependency review; expensive foundations for an elementary target |
| CS-C | Complex coordinates: replace squares with squared moduli and $C$ with $\sum\overline{x_i}y_i$; use a complex scalar residual or phase reduction | Extension of CS-Q/CS-N, not the real target; conjugation convention must be explicit | Prepare complex inner products and Crouzeix; prohibit the existing complex CS theorem |
| CS-W | Nonnegative weights: prove $(\sum w_i x_i y_i)^2\le(\sum w_i x_i^2)(\sum w_i y_i^2)$ | Generalization with $w_i\ge0$, zero weights allowed | Teach weighted geometry; equality constrains positive-weight support only |
| CS-E | Finite probability and covariance: expand variance of $X-tY$ or apply a completed finite weighted route to centered coordinates | Application/generalization; distinguish finite expectation from measure-theoretic expectation | Bridge to Bishop PRML and Deep Learning, with nonzero-variance hypotheses only for correlation |
| CS-H | Abstract symmetric positive-semidefinite real bilinear form: derive CS directly from positivity of $b(x-ty,x-ty)$ | Quadratic family; symmetry is required and degeneracy differs from an inner product | Teach which axioms are sufficient; do not infer vector dependence from equality for degenerate forms |
| CS-M | Integral version: nonnegativity of the integral of $(f-tg)^2$ with square-integrability assumptions | Quadratic family plus substantial measure theory | Later bridge to Spivak/analysis; audit product integrability proof to avoid using integral CS as a prerequisite |

An angle proof using $\cos\theta=\langle x,y\rangle/(\|x\|\|y\|)$ is not accepted
without independently constructing the angle and proving the ratio is in
$[-1,1]$. A proof via the norm triangle inequality also needs a circularity
audit, since that inequality is often established using CS. General Hölder,
Bessel, and operator-norm bounds are not acceptable substitutes for the
from-scratch core proof.

### Equality, strictness, and transfer contracts

First prove the division-free equality contract

$$C^2=AB\iff\forall i,j,\ x_i y_j=x_j y_i.$$

Use the derived double-sum identity and nonnegative-sum-zero reasoning.
Then prove the equivalent guarded proportionality statement
$x=0\ \lor\ \exists t\in\mathbb R,\ y=tx$.
Do not assert $\exists t,\ y=tx$ alone: it fails when $x=0$ and $y\ne0$.
With $x\ne0$, pick a nonzero coordinate and explicitly justify the division.
Derive strict inequality when some two-coordinate minor is nonzero.

Add the absolute-value/square-root statement as a named bridge, not a silent
replacement of the core target. For complex vectors use complex proportionality;
for weighted vectors use proportionality on positive-weight support. For
integrable functions equality is an almost-everywhere assertion. These are
different contracts, not cosmetic type changes.

### Lesson structure and original exercises

Each lesson has six visible stages: predict the equality case; explore a
two-coordinate example; state the key identity; prove each helper; assemble
the universal Lean theorem; explain a counterexample to a weakened or altered
claim. Keep prose derivations next to named declarations and link with stable
exercise IDs. Do not copy textbook exercises or their solutions.

| Lesson | Prerequisites | Original exercise and acceptance condition |
| --- | --- | --- |
| CS-00 | Finite functions, sums, powers, propositions | Spell the universal target without using its alias; explain why dimension zero must be included |
| CS-01 | Nonnegative sums and ordered fields | From $\sum y_i^2=0$ derive every coordinate zero; test both empty and nonempty index types |
| CS-02 | Polynomial expansion | Derive the two-variable defect identity $(a^2+b^2)(c^2+d^2)-(ac+bd)^2=(ad-bc)^2$ |
| CS-03 | CS-01 and sum distribution | Derive the parameterized quadratic, predict $t=C/B$, and justify the zero case before division |
| CS-04 | Double sums and reindexing | Build CS-L; explain why ordered pairs produce a factor of two |
| CS-05 | Square roots and homogeneity | Build CS-N; exhibit why $C\le1$ does not imply $C^2\le1$ |
| CS-06 | Finite-set induction | Build CS-I; show the mixed-term inequality and say where the induction hypothesis is used |
| CS-07 | CS-L and finite-sum-zero | Prove the minor equality criterion and repair the false unguarded proportionality claim |
| CS-08 | Coordinate geometry | Build the residual identity and identify its exact relationship to CS-Q |
| CS-09 | Nonnegative weights and finite probability | Derive covariance CS, then state the additional assumptions needed to define normalized correlation |
| CS-10 | Dependency tracing | Submit a renamed wrapper around library CS and explain why a name-only filter misses it |
| CS-11 | Generalization | Choose complex, weighted, or semidefinite form; list exactly which real-case hypotheses and equality claims change |

Hints form a ladder: suggest an equality pattern, then an intermediate lemma,
then a local algebra move. The final proof is a separate reveal. Exercise
stubs containing holes must be outside the accepted production import tree.
Do not count copying a revealed solution as independent completion.

Assess explanation quality separately with four human-reviewed dimensions:
identifies the key construction, accounts for hypotheses and exceptional
cases, relates the proof to another presentation, and diagnoses a false
variant. Score each 0/1/2 for absent/partial/complete reasoning. Kernel success
does not automatically award these points. Measure learning only through
consented, held-out pre/post and transfer exercises; no learning-effect claim
is supported by this design alone.

### Lean module boundaries

All paths below are relative to `formalization/lean/` and are planned unless
already present. Keep the namespace `TextbookBench`.

| File | Responsibility |
| --- | --- |
| `MathematicalFoundations/TextbookCauchySchwarzTarget.lean` | Move the existing target definition here without altering its type; no reference proof |
| `MathematicalFoundations/TextbookCauchySchwarz.lean` | Preserve the existing import path and `cauchySchwarzReference`; import the separated target |
| `MathematicalFoundations/CauchySchwarz/Elementary.lean` | Audited low-level shared sum and zero-square helpers |
| `MathematicalFoundations/CauchySchwarz/Quadratic.lean` | CS-Q construction and `cauchySchwarzQuadratic` |
| `MathematicalFoundations/CauchySchwarz/Lagrange.lean` | CS-L identity and `cauchySchwarzLagrange` |
| `MathematicalFoundations/CauchySchwarz/Normalization.lean` | CS-N construction and `cauchySchwarzNormalization` |
| `MathematicalFoundations/CauchySchwarz/Induction.lean` | CS-I finite-set induction and `cauchySchwarzInduction` |
| `MathematicalFoundations/CauchySchwarz/Equality.lean` | Minor, guarded proportionality, and strictness contracts |
| `MathematicalFoundations/CauchySchwarz/Controls.lean` | Separate universal type checks and boundary instantiations for all four proofs |
| `MathematicalFoundations/CauchySchwarz/Audit.lean` | Trusted proof-expression dependency/axiom inspection; not candidate-controlled acceptance |
| `MathematicalFoundations.lean` | Import accepted controls only after the corresponding route and audit pass |

Do not make core route modules import one another or the reference module.
Learning modules may import the core they explain, but must be classified as
applications, not independent proofs. Narrow Mathlib imports reduce accidental
access; they do not prove absence of indirect high-level dependencies.

### Verification design

1. **Statement gate.** A trusted checker loads the immutable target and checks
   each candidate's declaration against it using Lean type checking. Inspect
   declaration kind and universe parameters. Reject extra hypotheses and
   weaker types; do not accept a same-named declaration as evidence.
2. **Kernel and axiom gate.** Require successful compilation and inspect
   transitive axioms. Reject `sorryAx`, new axioms, and trust bypasses. Record
   any permitted foundational axioms such as `propext`, `Classical.choice`,
   and `Quot.sound` individually. Do not claim their use invalidates an
   ordinary Mathlib theorem. `#print axioms` is necessary but cannot establish
   the dependency restriction by itself.
3. **Dependency gate.** Traverse constants in proof expressions and the types
   and available values of referenced declarations recursively, memoizing
   visited names. Reject unavailable bodies outside the reviewed trust boundary. Bind the
   audited environment to source/toolchain/dependency digests. Maintain a
   reviewed low-level boundary and exact prohibited declarations for this
   pinned Mathlib revision. Reject unknown dependencies pending review, not
   just known spelling matches. Include reference controls and aliases in
   negative tests. A denylist is a regression aid, not a semantic oracle for
   all possible reformulations of CS.
4. **Route gate.** Review named identities and import edges. No CS core proof
   may depend on another core conclusion. Permit basic shared algebra only.
   A machine cannot establish mathematical independence merely by comparing
   theorem names or proof-term hashes; human review owns the family label.
5. **Control gate.** Instantiate the exact universal theorem at `Fin 0`,
   singleton vectors, both choices of zero vector, negative proportional
   vectors, and nonproportional vectors. These are regression controls, not
   a substitute for the universal proof.
6. **Execution gate.** Treat local Lean builds as non-hermetic. Untrusted
   learner/agent source may execute elaborator code; kernel checking alone
   does not sandbox it. Production acceptance additionally requires an
   independently qualified confined runner with no network and read-only
   trusted roots. On macOS this means the reviewed Seatbelt boundary, not a
   temporary directory alone. Do not execute adversarial source on the host
   before that boundary is qualified.

Record route ID, target digest, source digest, Lean and Mathlib revisions,
dependency-policy digest, audited constant set digest, axiom set, argv/cwd,
environment policy, exit status, elapsed time, timeout, stdout/stderr digests,
and execution classification. An absent field is not a successful check.
Separate `proved`, `policy_rejected`, `compile_error`, `timeout`, and
`infrastructure_blocked`; a timeout is not mathematical falsity.

### Adversarial and boundary acceptance matrix

All intentionally invalid candidates live in disposable test inputs, never
in the production import tree. Compile success and policy success are distinct.

| Case | Expected outcome |
| --- | --- |
| Each of the four complete universal proofs | Kernel and dependency policy pass |
| Empty dimension; either/both vectors zero | Universal specialization passes without new hypotheses |
| $x=(1,-2)$, $y=(-3,6)$ | Equality passes despite negative proportionality |
| $x=(1,0)$, $y=(0,1)$ | Strict inequality control passes |
| $(\sum x_i y_i)^2\ge AB$ as universal target | Statement gate rejects; preceding orthogonal vectors refute it |
| Universal strict inequality | Statement gate rejects; zero/proportional vectors refute it |
| Target restricted to nonnegative coordinates or positive dimension | Statement gate rejects extra hypotheses |
| Closed numerical instance presented as universal proof | Statement gate rejects |
| Existing Mathlib CS invoked directly | Kernel may pass; dependency gate rejects |
| Library CS behind a renamed helper in a separate module | Transitive dependency gate rejects |
| Reference proof reached through simplification/automation | Dependency gate rejects the emitted proof, regardless of tactic spelling |
| Another approved core proof used to implement a route | Route independence gate rejects |
| `sorry` hidden in a helper | Axiom gate rejects |
| A user axiom asserting CS | Axiom gate rejects |
| Altered target constant with the trusted name | Trusted-environment/type gate rejects |
| Precompiled output from changed source or dependency policy | Digest/provenance gate rejects |
| Nonterminating elaborator | Qualified runner returns timeout and terminates descendants |
| Filesystem write or network attempt by elaborator | Qualified sandbox denies; retain denial evidence |
| Missing warm dependency artifact | Infrastructure blocked; no implicit cache download |
| Unguarded $y=tx$ equality claim when $x=0,y\ne0$ | Counterexample test refutes it |
| Negative weight in weighted extension | Reject omitted nonnegativity hypothesis; $w=(1,-1),x=(1,0),y=(0,1)$ gives $0\nleq-1$ |
| Complex square used instead of modulus square | Statement gate rejects wrong type/target |

### Delivery waves and decision gates

| Wave | Deliverable | Exit evidence |
| --- | --- | --- |
| E0 | This research/design, approved target and dependency boundary | Source access labels, reviewed mathematical sketches, no implementation claim |
| E1 | Target separation, elementary helpers, CS-Q, trusted controls | Missing-declaration red test followed by exact universal Lean proof; reference compatibility preserved |
| E2 | CS-L, equality, strictness | Double-sum identity checked; equality counterexample controls; independent dependency audit |
| E3 | CS-N and CS-I | Two more universal proofs; no core conclusion reused; square-root and insertion obligations explicit |
| E4 | Trusted local dependency-audit tool and complete positive/negative policy fixtures | Direct, aliased, and transitive shortcut rejection; no unsupported claim of hostile-source containment |
| E5 | Original lessons, staged hints, transfer exercises, coverage links | Every accepted solution linked to a declaration and verifier receipt; explanation reviewed separately |
| E6 | Projection, Gram, weighted variance/Jensen presentations | Dependency-family labels and exact support hypotheses; count as presentations/applications unless independently justified |
| E7 | Weighted, complex, semidefinite and finite probability extensions | New targets and equality contracts reviewed individually; no blanket inheritance of real-case acceptance |
| E8 | Integral route and qualified untrusted execution | Integrability/circularity review plus separately tested sandbox and reproducible receipts |

E1 through E3 may compile locally while E4 is built, but are not accepted as
policy-compliant educational artifacts until E4 passes. E5 can use reviewed
trusted proofs; public untrusted submission execution waits for E8's runner.
Each implementation slice pairs its failing contract test with the repair
before landing. No knowingly failing future-wave test enters the accepted
aggregate build.

Use the pinned repository toolchain and the verified canonical warm cache.
Do not run `lake update`, cache hydration, or change Mathlib during iteration.
Coordinate shared-output build windows with other worktrees before any Lean
gate. Run narrow owned modules during iteration, then `mise run
lean-foundations`, wrapper regressions, and finally `mise run verify` on the
frozen candidate before committing or landing. When direct Mathlib imports
change, update all three existing fake-cache rosters in
`crates/harp/tests/lean_library.rs` and
`crates/harp/tests/mathematical_foundations_lean.rs`; retain negative missing-cache
tests. Regenerate Atlas outputs with canonical prose, then refresh the import
receipt using the verifier's reported digest.

### Relationship to the wider curriculum

Use this laboratory as a small test of the teaching/verification contract,
not a replacement for the all-chapter inventory. Preserve Lax's expanded
second edition from 2007, Spivak's *Calculus on Manifolds*, Bishop's PRML from
2006, and Bishop and Bishop's *Deep Learning* as separate book tracks.
Attach source locators only after checking the relevant edition; do not invent
exercise numbers. Map Lax to projections/Gram forms, Bishop to covariance and
least squares, and Spivak to analytic generalization with the appropriate
function-space prerequisites. Formalized textbook statements, original
teaching exercises, applications, empirical algorithms, and prose-only claims
must remain distinct inventory categories. No grand Crouzeix claim follows
from completion of this laboratory.

### Research provenance and limitations

The proof sketches and educational choices above are newly authored design
reasoning, not quoted solutions. Source links identify mathematical
orientation, not evidence of Lean execution. Web observations are dated
2026-09-08 UTC and are not immutable captured artifacts. No source PDFs or textbook
solutions are redistributed.

- [Steele's author page](https://www-stat.wharton.upenn.edu/~steele/Publications/Books/CSMC/CSMC_index.html)
  identifies sample chapters and original sources. Search-index text was
  accessible; direct page retrieval timed out or failed TLS. Do not mark its
  linked chapter PDFs or original papers as read.
- [Cambridge's contents](https://www.cambridge.org/core/books/abs/cauchyschwarz-master-class/contents/1906D4D36FD67BEFD6510C63FFB287C9)
  confirms the book's chapters on Cauchy, AM-GM, Lagrange identity, geometry,
  convexity, and integrals. Chapter titles alone do not establish the exact
  proof presented in each chapter. The publisher's sample PDF also failed
  direct retrieval in the initial pass.
- [W. T. Gowers, A tiny remark about the Cauchy-Schwarz inequality](https://www.dpmms.cam.ac.uk/~wtg10/csineq.html)
  was read in full. It motivates the sum of squared minors from the equality
  condition, develops a scaled-vector argument, and discusses complex phase
  adjustment and the residual-minimization interpretation. It supports the
  discovery-first lesson structure. Our unrestricted squared target requires
  explicit zero cases and differs from a signed upper-bound equality claim.
- [Purdue, Section 6.7 Inner Product Spaces](https://www.math.purdue.edu/~xu1121/Sec6.7)
  was read in full. Its Theorem 16 proves CS through a line projection and a
  right-triangle bound. Our proposed reconstruction replaces that geometric
  bound with a locally derived residual-square identity. The source supports
  the projection presentation, not a claim that its prerequisites are
  automatically independent in Mathlib.
- [Frank R. Kschischang, Useful Inequalities from Jensen to Young to Hölder to Minkowski](https://www.comm.utoronto.ca/frank/notes/ineq.pdf)
  is a University of Toronto note dated 2019-09-25. The research review read
  its finite Jensen induction, Young, normalized Hölder, and complex CS
  development. It supports that teaching sequence, not permission to invoke
  Hölder in the core exercise. The directed scalar-multiple equality wording
  requires a zero-vector guard for our target.

The bounded background review admitted six primary endpoints: Steele,
Gowers, Purdue, Toronto, IIT Madras orthogonality notes, and Heidelberg applied
mathematics notes. Three were fully readable: Gowers, Purdue, and Toronto.
The [IIT Madras notes](https://ee.iitm.ac.in/~uday/notes/linalg/orth.html) and
[Heidelberg notes](https://simweb.iwr.uni-heidelberg.de/~gkanscha/notes/B.pdf)
were not readable in that pass and support no promoted claim. The Cambridge
contents were checked separately during the initial design. There is no
inspected source in this bounded review for the full optimization or integral
construction; those remain explicitly authored design proposals. Exact
Steele chapter-level proof correspondence remains an open source-verification
step, not a reason to fabricate an attribution.

### Implemented core and teaching entry points

The four core routes, equality/strictness contracts, and local dependency
checks are implemented. All four routes have compiled against the pinned
Lean 4.32.1 and Mathlib revision
`520045ab14e26149ee970e2e617ca04b09bde5d6`. These are local compile results,
not hermetic execution receipts. Kata issue `p52e` records gate and review
evidence for the implementation; a source document is not a release receipt.

| Lesson | Declaration in namespace `TextbookBench` | What to inspect |
| --- | --- | --- |
| CS-00 | `CauchySchwarzTarget` | Universal quantifiers, squared real target, no positivity assumption |
| CS-01 | `Elementary.zero_of_sum_sq_zero` | Every square is bounded by its nonnegative sum |
| CS-03 | `cauchySchwarzQuadratic` | Local `expand`, explicit zero branch, substitution before cancellation |
| CS-04 | `lagrangeExpansion`, `cauchySchwarzLagrange` | Derive the identity before applying nonnegativity |
| CS-05 | `cauchySchwarzNormalization` | Local `hu`, `hv`, `plus`, `minus`, both bounds, final `pairing` |
| CS-06 | `cauchySchwarzInduction` | Local insertion hypothesis, `hid`, `hcross`, and defect update |
| CS-07 | `cauchySchwarzEqualityIffMinors`, `cauchySchwarzEqualityIffProportional`, `cauchySchwarzStrictOfMinor` | Zero-safe equality and strictness from a nonzero minor |
| CS-10 | `Audit.audit`, `Audit.fromFoundations` | Type check, recursive type/body traversal, fixed foundation boundary, separate axioms |

Start with the original exercise prompts above without reading the code.
For the quadratic route, the first hint is to measure the squared error of
approximating $x$ by a multiple of $y$. The second hint is to expand the
parameterized sum. The final hint is to eliminate its linear term by choosing
$t=C/B$, after treating $B=0$. The named theorem is the solution reveal.

For Lagrange, ask which expressions vanish for proportional coordinates;
then square and sum those expressions; only then expand. For normalization,
first remove scale, then compare the sum and difference, and finally explain
why both comparisons are needed. For induction, first write the defect after
insertion; isolate its mixed term; then use the scalar squared identity
displayed in CS-I. These hint sequences are newly authored teaching material.

As a transfer exercise, compare $x=(1,-2)$ with $y=(-3,6)$: the squared
inequality is an equality even though the pairing is negative. Contrast
$x=(1,0)$, $y=(0,1)$, where the pairing vanishes but the inequality is strict.
Explain why $x=0$, $y=(1)$ refutes the unguarded assertion that equality always
means $y=tx$. The compiled controls include these distinctions. This tests
mathematical interpretation, not just command completion.

The accepted import path is
`MathematicalFoundations.CauchySchwarz.Accepted`; the foundations aggregate
imports it. It checks the four exact core targets and rejects cross-route
reuse. The old `cauchySchwarzReference` remains a clearly separated reuse
control. Equality lemmas and concrete controls are kernel-compiled and
source-reviewed; the closed four-route dependency-policy result is not a
claim that every later exercise has the same audited dependency set.

`FoundationPolicy.lean` contains 391 explicitly named low-level foundation
roots and 11 owned declarations. Its authorization is their transitive closure
in the trusted pinned environment, not a policy learned from the candidate at
acceptance time. The full closure is traversed, including theorem bodies;
unknown imported dependencies outside this boundary fail closed. Review any
boundary change and recheck the pinned environment. Root count is not proof
size, and neither count measures student understanding.

Run `mise run lean-foundations` to compile the positive controls and policy
checks. Run `mise run lean-cauchy-audit` to also execute the trusted negative
fixtures outside the production import tree. `mise run verify` includes those
regressions after the aggregate Lean build. The negative fixtures contain
intentional rejected assertions; they are not accepted library theorems.
No user-submitted source should be passed to this local regression runner.

The larger E6-E8 presentations, generalizations, and qualified hostile-source
runner remain future waves. CS-02 and the later transfer lessons remain
exercise/design entries unless a separate declaration is listed. No new
dependency downloads or GitHub publication are implied by this implementation.
