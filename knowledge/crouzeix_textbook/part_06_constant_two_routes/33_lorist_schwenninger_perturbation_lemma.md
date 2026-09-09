---
id: cft-chapter-33-lorist-schwenninger-perturbation-lemma
title: The Lorist–Schwenninger perturbation lemma
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-26
tags: [crouzeix-textbook, constant-two-routes, mathematics, lean]
confidence: high
canonical: 33_lorist_schwenninger_perturbation_lemma.md
chapter: 33
part: 6
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 33: The Lorist–Schwenninger perturbation lemma

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-vi-constant-two-routes|Part VI — Constant-two routes]]
Previous: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/32_jin_constant_two_endpoint|Chapter 32 — Jin’s constant-two endpoint]]
Next: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/34_lorist_schwenninger_realization|Chapter 34 — The Lorist–Schwenninger realization]]

## Opening problem

Can one reach the constant two without constructing a positive-real
completion? The Lorist--Schwenninger route says yes. It retains every power in
a single dilation, compares adjacent powers, and converts the operator problem
into a scalar recurrence. If `κ = ‖T‖` were larger than two, two independently
derived bounds on one displacement would have incompatible signs.

## Conceptual model

The dilation data writes a doubled compressed adjoint power as `(T*)^n + E_n`.
The perturbations `E_n` are uniformly bounded and commute with `T`. In finite
dimension, choose a unit vector `x` attaining `‖T‖ = κ`; it obeys the
singular-vector equation `T*T x = κ²x`. Real parts of inner products involving
`E_n x` and `(T*)^n x` then form a bounded scalar sequence. The proof studies
its one-step drift rather than any one power in isolation.

## Notation ledger

Mathlib's complex inner product is **conjugate-linear in its first argument and linear in its second**.
Operator-theory texts often use the opposite ordering.
The provider therefore represents the source scalar by the conjugate-swapped
expression `Re ⟪x, E_n T^n x⟫`; real parts agree under this swap.

| Mathematics | Role | Maintained Lean name |
|---|---|---|
| `E`, `K` | finite-dimensional target Hilbert space and dilation Hilbert space | type parameters of `LoristSchwenninger.DilationData` |
| `T` | target operator on `E` | `DilationData.T` |
| `V`, `Q` | isometry into `K` and contraction on `K` | `DilationData.V`, `DilationData.Q` |
| `E_n` | uniformly bounded perturbation commuting with `T` | `DilationData.perturbation` at `n` |
| `κ = ‖T‖` | candidate operator norm | the real parameter `κ`, instantiated by `‖data.T‖` |
| unit `x` | norm-attaining right singular vector | `exists_unit_norm_attaining_and_adjoint_apply` |
| `T*T x = κ²x` | singular-vector equation | hypothesis `hsingular` |
| `d = Q*VTx - κVx` | common adjacent-power displacement | `DilationData.displacement` at `κ,x` |
| `b = ‖d‖²` | nonnegative displacement scalar | `DilationData.displacementSq` at `κ,x` |
| `m_n = Re ⟪E_n x, (T*)^n x⟫` | adjacent-power recurrence scalar | `recurrenceScalar` at `data,x,n` |
| first moment `m_1` | scalar shared by Equations (3) and (4) | `DilationData.firstPerturbationMoment` at `x` |

The last two rows use equal real parts in the two inner-product orientations;
the exact adjoint transfer is implemented in
`formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean`.

## Formal development

The pinned source locators for this workshop are literal coordinates into
`arxiv:2608.03841v1`: Lemma 1 is
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99`, the recurrence through
Equation (3) is
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L74-L90`, and Equation (4) plus
the scalar contradiction is
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98`.

**SOURCE CLAIM — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-006|CFT-CL-006]].**
Lorist and Schwenninger state the perturbation lemma and its
constant-two conclusion in the pinned arXiv v1 source. This is an attribution
to that versioned source, not a priority or peer-review claim.

**EVIDENCE — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-003|CFT-CL-003]].**
Harp records the exact source coordinates above and maintains a
locally compiled Lean provider for the recurrence, Equations (3) and (4), the
scalar endpoint, and operator assembly. A compiler receipt establishes only
the named declarations under their explicit hypotheses.

**TEXTBOOK EXPOSITION.** The notation ledger, decomposition into six cards,
examples, ML analogy, and exercises below are Harp's teaching organization.
They are not source prose. CFT-33-001 and CFT-33-002 are now reconstructed
line by line and matched to compiled textbook theorems. CFT-33-003 reconstructs
the separate finite telescoping and limiting step; CFT-33-004 and CFT-33-005
derive Equation (4) and close the scalar contradiction. CFT-33-006 assembles
the norm-attainment branch, both inequalities, and the scalar endpoint.

### CFT-33-001 — recurrence difference identity {#cft-33-001}

#### Purpose

**Motivation.** Expose the exact adjacent-power equality before any inequality
or division. Every later estimate acts on this identity, so a hidden sign or
inner-product convention here would propagate through the whole route.

#### Statement

Let `data` be source-faithful dilation data, let `κ : ℝ`, `n : ℕ`, and `x∈E`,
and assume `T*Tx=κ²x`. For `y_n = (T*)^n x`, the checked provider states

```text
κ m_n - m_{n+1}
= (κ²-κ) ‖y_n‖² - Re ⟪adjacentPowerDefect κ n x, y_n⟫.
```

#### Hypothesis ledger

The identity uses the dilation identity, commutation of `E_n` with `T`, and
`T*T x = κ²x`. It does not yet require `κ > 1` or `‖x‖ = 1`.

#### Proof roadmap

Rewrite both recurrence scalars, transfer `T` through the inner product, use
commutation, insert the singular-vector equation, and collect the defect.

#### Proof

Put `y_n=(T*)^n x`, and abbreviate the doubled compressed adjoint power by
`S_n*`. The perturbation identity is `E_n=S_n*-(T*)^n`. Start with the exact
Lean definition, not the already-rewritten form:

```text
m_n = Re ⟪x, (E_n T^n)x⟫.
```

Because `E_n` commutes with `T`, it also commutes with `T^n`:

```text
E_n T^n = T^n E_n.
```

Substitute that operator identity, transfer `T^n` across the inner product by
adjunction, and finally use symmetry of the real part. Keep the transformations
as separate displayed equalities:

```text
m_n
= Re ⟪x, T^n(E_nx)⟫
= Re ⟪(T*)^n x, E_nx⟫
= Re ⟪E_nx, (T*)^n x⟫
= Re ⟪E_nx, y_n⟫.
```

The same bridge is required at the successor index; it is not an implicit
change of notation. Begin again with the definition and the powered
commutation identity

```text
m_{n+1} = Re ⟪x, (E_{n+1} T^(n+1))x⟫.
E_{n+1} T^(n+1) = T^(n+1) E_{n+1}.
```

then repeat the two inner-product transformations:

```text
m_{n+1}
= Re ⟪x, T^(n+1)(E_{n+1}x)⟫
= Re ⟪(T*)^(n+1)x, E_{n+1}x⟫
= Re ⟪E_{n+1}x, (T*)^(n+1)x⟫.
```

This is exactly the mathematical content of the provider's private
`recurrenceScalar_eq_inner` bridge at `n` and at `n+1`. The commutations in
the proof have distinct roles:

| role | operator identity |
|---|---|
| scalar bridge at `n` | `E_n T^n = T^n E_n` |
| scalar bridge at `n+1` | `E_{n+1} T^(n+1) = T^(n+1) E_{n+1}` |
| adjacent recurrence step | `T E_{n+1}x = E_{n+1}Tx` |

Now peel the final copy of `T*` from the second slot and transfer it to a copy
of `T` in the first slot:

```text
m_{n+1}
= Re ⟪E_{n+1}x, T* y_n⟫
= Re ⟪T E_{n+1}x, y_n⟫.
```

Here the second equality is valid with Mathlib's convention because
`⟪u,T*v⟫=⟪Tu,v⟫`. Apply the adjacent-recurrence commutation from the ledger:

```text
T E_{n+1}x = E_{n+1}Tx.
```

The singular-vector equation propagates through the preceding `n` adjoint
powers, so in particular `(T*)^(n+1)Tx = κ²y_n`:

```text
(T*)^(n+1)Tx
= (T*)^n(T*Tx)
= (T*)^n(κ²x)
= κ²y_n.
```

Define the adjacent defect, exactly as in Lean, by

```text
z_n = adjacentPowerDefect κ n x = (S_{n+1}* T-κ S_n*)x.
```

Substituting `E_n=S_n*-(T*)^n` and the propagated singular-vector identity
gives the vector equality

```text
κ E_nx-E_{n+1}Tx
= κ(S_n*x-y_n)-[S_{n+1}*Tx-κ²y_n]
= (κ²-κ)y_n-z_n.
```

Taking the real inner product with `y_n` finishes the calculation:

```text
κ m_n - m_{n+1}
= Re ⟪κ E_nx-E_{n+1}Tx,y_n⟫
= Re ⟪(κ²-κ)y_n-z_n,y_n⟫
= (κ²-κ) ‖y_n‖²-Re ⟪z_n,y_n⟫.
```

This is the displayed statement, with `z_n=adjacentPowerDefect κ n x`.

**Worked instance.** At the scalar-algebra level, take `κ=2`, `‖y_n‖=1`,
and `z_n=0`. The identity reads `2m_n-m_{n+1}=2`. This does not assert that
arbitrary dilation data realize those values; it checks the coefficient
`κ²-κ=2` and the sign of the defect term before inequalities are introduced.

#### Boundary case

The denominator-free equality still makes sense at `κ = 0` and `κ = 1`.

#### Pedagogical prerequisites

CFT-23-004, CFT-23-005, and the common route entry CFT-29-002.

#### Lean correspondence

The exact public theorem
`CrouzeixTextbook.Part06.recurrence_difference_identity` is in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]].
It reexports the same normalized proposition proved by
`DilationData.recurrence_difference_identity` in
[[formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean|OperatorRecurrence.lean]].
The generated compiler correspondence checks both declarations.

**Receipt audit.** The public declaration is at
`formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:18:9`; the provider
is at
`formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean:41`.
The formal mode is `reexported-proof`. Direct kernel dependency:
`CrouzeixConjecture.LoristSchwenninger.DilationData.recurrence_difference_identity`.

The fresh receipt records normalized type fingerprint
`3ab6cb427af92fba8115e75e77b5bc71e2f0780aafd8a71d60de7e88851a7921`,
reported axioms `Classical.choice`, `Quot.sound`, and `propext`, and verification
target `CrouzeixTextbook`.

#### Historical context

See the labeled source/evidence/textbook boundary above and source Lemma 1 at
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99`.

#### ML analogy

**Mathematical object / ML counterpart.** The exact adjacent-power recurrence
is compared with the one-step drift of a recurrent linearization at one fixed
Jacobian.

**Exact transfer.** Once the fixed linear maps and defect vector are supplied,
the displayed equality is ordinary inner-product algebra and transfers
without approximation.

**Non-transfer.** A stochastic or time-varying Jacobian does not supply one
commuting power family or one fixed defect; extra pathwise identities and
uniform bounds would be needed.

**Diagnostic.** Compute both sides for one index before estimating either
side. Reject the analogy if the measured residual is not the same vector as
the defect in the equality.

### CFT-33-002 — recurrence difference lower bound {#cft-33-002}

#### Purpose

**Motivation.** Turn the defect pairing into a scalar lower bound uniform in
the power index. This is the step that makes all adjacent powers summable by
one scalar certificate.

#### Statement

Let `data`, `κ`, `n`, and `x` be as in CFT-33-001, retain
`T*Tx=κ²x`, and assume `κ>1`. Then

```text
-‖displacement κ x‖²/(κ²-κ) ≤ κ m_n - m_{n+1}.
```

#### Hypothesis ledger

All hypotheses of CFT-33-001 plus `κ > 1`; positivity of `κ²-κ` is essential.

#### Proof roadmap

Factor the adjacent defect through `d`, bound it using the contraction and
isometry, then complete a real quadratic square.

#### Proof

Set

```text
a = κ²-κ,   u = y_n = (T*)^n x,
z = adjacentPowerDefect κ n x,   d = displacement κ x.
```

Because `κ>1`, we have `a=κ(κ-1)>0`. CFT-33-001 says that the
recurrence difference is `a‖u‖²-Re ⟪z,u⟫`. Expand the square using a real
positive scalar `a`:

```text
a‖u - z/(2a)‖²
= a[‖u‖² - (1/a)Re ⟪z,u⟫ + ‖z‖²/(4a²)]
= a‖u‖² - Re ⟪z,u⟫ + ‖z‖²/(4a).
```

Equivalently, the exact completed-square identity is

```text
a‖u - z/(2a)‖² - ‖z‖²/(4a)
= a‖u‖²-Re ⟪z,u⟫.
```

The squared norm and `a` are nonnegative, so discarding the first term gives

```text
-‖z‖²/(4a) ≤ a‖u‖²-Re ⟪z,u⟫ = κ m_n-m_{n+1}.
```

The dilation identity does not merely assert the norm estimate. It gives the
exact factorization

```text
z_n = 2 V* (Q*)^n d.
```

Here `V` is an isometry, so its adjoint `V*` is a contraction. The operator
`Q` is a contraction by the dilation data, hence `Q*` and every power
`(Q*)^n` are contractions. Apply those facts one at a time:

```text
‖z_n‖ = 2‖V*(Q*)^n d‖
      ≤ 2‖(Q*)^n d‖
      ≤ 2‖d‖.
```

Thus, in the abbreviations of this card, `‖z‖ ≤ 2‖d‖`.

Squaring preserves this inequality. Since `4a>0`, division and multiplication
by `-1` reverse its direction:

```text
-‖d‖²/a ≤ -‖z‖²/(4a).
```

Combining the last two displays and restoring `a=κ²-κ` yields

```text
-‖d‖²/(κ²-κ) ≤ κ m_n-m_{n+1}.
```

**Worked instance.** For a one-dimensional scalar check, take real coordinates
`u=s` and `z=t`.
Then `a(s-t/(2a))²-t²/(4a)=as²-ts`, exactly the claimed expansion.
Equality in the final bound requires both `u=z/(2a)` (the discarded square
vanishes) and `‖z‖=2‖d‖` (every contraction used by the defect factorization
is saturated). Conversely, those two conditions force equality. In the
special case `d=0`, they reduce to `z=0` and `u=0`.

#### Boundary case

At `κ = 1` the displayed denominator vanishes, so this form cannot be used.

#### Pedagogical prerequisites

CFT-09-006 and CFT-33-001.

#### Lean correspondence

The exact public theorem
`CrouzeixTextbook.Part06.recurrence_difference_lower_bound` is in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]].
Its provider theorem is in
[[formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean|OperatorRecurrence.lean]],
and the reusable square identity is in
[[formalization/lean/Crouzeix/LoristSchwenninger/CompletedSquare.lean|CompletedSquare.lean]].
The two intervening operator facts are
`DilationData.adjacentPowerDefect_factorization` and
`DilationData.adjacentPowerDefect_norm_le` in
[[formalization/lean/Crouzeix/LoristSchwenninger/Dilation.lean|Dilation.lean]].

**Receipt audit.** The public declaration is at
`formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:32:9`; the provider
is at
`formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean:80`.
The formal mode is `reexported-proof`. Direct kernel dependency:
`CrouzeixConjecture.LoristSchwenninger.DilationData.recurrence_difference_lower_bound`.

The fresh receipt records normalized type fingerprint
`4a8ff93fdef7e06d40d043d6482afd1e7dc887b03c3c5cba1d09447a22d174c5`,
reported axioms `Classical.choice`, `Quot.sound`, and `propext`, and verification
target `CrouzeixTextbook`.

#### Historical context

**SOURCE CLAIM — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-006|CFT-CL-006]].**
The source recurrence and Equation (3) occupy the exact locator
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L74-L90`.

**EVIDENCE — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-003|CFT-CL-003]].**
Harp's local compiler receipt checks the named theorem and its stated
hypotheses. Local compilation does not establish peer review, acceptance, or
journal publication, and the source attribution does not establish historical
priority.

#### ML analogy

**Mathematical object / ML counterpart.** The completed-square lower bound is
compared with a Lyapunov drift floor controlled by a fixed residual energy.

**Exact transfer.** The real quadratic inequality transfers exactly to any
fixed feature-space vectors with `a>0` and the same norm and inner product.

**Non-transfer.** The operator factorization `‖z‖≤2‖d‖` and exact commutation
are stronger than typical learned-dynamics assumptions; they need separate
proofs in an ML model.

**Diagnostic.** Evaluate the discarded square and the factorization slack
separately. Equality is plausible only when both vanish.

### CFT-33-003 — equation three lower bound {#cft-33-003}

#### Purpose

**Motivation.** Convert all one-step recurrence inequalities into a bound on
the first moment. This is where a local drift bound becomes the source's
global Equation (3).

#### Statement

Let `data` be source-faithful dilation data, assume `κ>1`, `‖x‖=1`, and
`T*Tx=κ²x`, and put `b=‖displacement κ x‖²`. Source Equation (3) is

```text
-b/[κ(κ-1)²] ≤ Re ⟪x, E_1 T x⟫.
```

#### Hypothesis ledger

CFT-33-002, `‖x‖ = 1`, the singular-vector equation, and the uniform bound on
`m_n` supplied by the perturbation and power bounds.

#### Proof roadmap

Divide by inverse powers of `κ`, sum a finite recurrence, control the terminal
term uniformly, and pass to the geometric-series limit.

#### Proof

Set

```text
b = ‖d‖²,
r_n = -b/(κ²-κ),
m_n = Re ⟪x, E_n T^n x⟫ = Re ⟪E_n x, (T*)^n x⟫.
```

CFT-33-002 supplies the pointwise inequality, for every `n ≥ 1`,

```text
r_n ≤ κm_n-m_{n+1}.
```

Make the uniform hypotheses explicit. The dilation data names

```text
B = data.bound ≥ 0,
‖E_n‖ ≤ B   for every n.
```

These are the fields `DilationData.bound_nonneg` and
`DilationData.perturbation_norm_le` in
[[formalization/lean/Crouzeix/LoristSchwenninger/Dilation.lean|Dilation.lean]].
The already established target-power estimate follows from the same data. The
identity `E_n=2V*(Q*)^nV-(T*)^n` rearranges to
`(T*)^n=2V*(Q*)^nV-E_n`. Since `V` is an isometry and `Q` is a contraction,
`‖V‖=‖V*‖=1` and `‖(Q*)^n‖≤1`; the triangle and product inequalities give

```text
‖(T*)^n‖ ≤ 2‖V*‖ ‖(Q*)^n‖ ‖V‖ + ‖E_n‖ ≤ 2+B.
```

Taking adjoints preserves operator norm, so

```text
‖T^n‖ = ‖(T*)^n‖ ≤ 2+B.
```

In particular, the second uniform hypothesis used below is `‖T^n‖ ≤ 2+B` for
every `n`.

This is the proof exposed as
`DilationData.target_power_norm_le_two_add_bound` in
[[formalization/lean/Crouzeix/LoristSchwenninger/Dilation.lean|Dilation.lean]].
Now Cauchy--Schwarz, the operator-norm inequality, and `‖x‖=1` yield the full
uniform chain

```text
|m_n| ≤ ‖x‖ ‖E_n T^n x‖ ≤ ‖E_n‖ ‖T^n‖ ≤ B(2+B).
```

Thus name `M = B(2+B) = data.bound * (2 + data.bound)` and conclude
`|m_n| ≤ M` for every `n`. The compiled statement is
`recurrenceScalar_abs_le` in
[[formalization/lean/Crouzeix/LoristSchwenninger/RecurrenceScalar.lean|RecurrenceScalar.lean]].
This single `M`, independent of `n`, is exactly what bounds the moving terminal
index below; pointwise finiteness alone would not suffice.

Now audit every denominator. From `κ>1` we obtain `κ>0`, hence `κ≠0`; also
`κ-1>0`, hence `κ-1≠0`. Therefore
`κ²-κ = κ(κ-1)>0`, and it too is nonzero. For every `n`, positivity is
preserved by powers, so `κ^(n+1) > 0` and `κ^(n+1) ≠ 0`. Division by this
positive power is legal and preserves the inequality direction:

```text
r_n/κ^(n+1) ≤ m_n/κ^n - m_{n+1}/κ^(n+1).
```

Fix `N ≥ 1` and sum this displayed inequality for `n=1,...,N`. Write the finite
statement before taking any limit:

```text
∑_{n=1}^N r_n/κ^(n+1)
≤ ∑_{n=1}^N [m_n/κ^n - m_{n+1}/κ^(n+1)]
= m_1/κ - m_{N+1}/κ^(N+1).
```

In particular, the finite inequality is

```text
∑_{n=1}^N r_n/κ^(n+1) ≤ m_1/κ - m_{N+1}/κ^(N+1).
```

Every intermediate term cancels: the negative
`-m_{n+1}/κ^(n+1)` from index `n` cancels the positive term with the same
index and denominator at index `n+1`. Only the explicitly displayed endpoints
remain. This finite cancellation is the content of
`Exercises.Chapter33.exercise_03_solution`; it is separate from the provider's
infinite-limit theorem.

The uniform bound controls the terminal endpoint:

```text
|m_{N+1}/κ^(N+1)| ≤ M/κ^(N+1).
```

Indeed the denominator is positive, so the absolute-value estimate may be
divided by it. Put `q=1/κ`. The inequalities `κ>1` and `κ>0` give `0<q<1`.
Thus `q^(N+1)→0`, and multiplication by the fixed finite constant `M` gives

```text
M/κ^(N+1) → 0.
```

The squeeze estimate therefore sends the terminal endpoint to zero. The same
condition `0<q<1` is exactly the convergence criterion for the remaining
geometric series. Its first term is `q²` and its ratio is `q`, so its sum is
`q²/(1-q)`. Simplifying that quotient gives

```text
∑_{n=1}^∞ 1/κ^(n+1) = 1/[κ(κ-1)].
```

Because `r_n=-b/(κ²-κ)` is constant, the finite left side is that constant
times the finite geometric sum. Let `N→∞` in the finite inequality. Continuity
of addition and multiplication, the terminal decay, and the evaluated series
give

```text
[-b/(κ²-κ)] * 1/[κ(κ-1)] ≤ m_1/κ.
```

Multiplication by the positive number `κ` preserves the direction. Finally use
`κ²-κ = κ(κ-1)` and the definition
`m_1 = Re ⟪x, E_1 T x⟫`:

```text
-b/[(κ²-κ)(κ-1)]
= -b/[κ(κ-1)²]
≤ Re ⟪x, E_1 T x⟫.
```

Thus, in the source's displayed form,
`-b/[κ(κ-1)²] ≤ Re ⟪x, E_1 T x⟫`.

**Worked instance.** For a scalar check, take `κ=2` and `m_n=1`. The finite telescoping sum is
`∑_{n=1}^N 1/2^(n+1)=1/2-1/2^(N+1)`, exactly the two endpoint terms above.
At `κ = 1`, by contrast, `κ²-κ` and `κ-1` vanish, the inverse-power ratio is
one, the geometric series diverges, and the terminal bound need not decay.
The displayed proof is therefore unavailable at that boundary.

#### Boundary case

The argument needs `κ > 1` for legal division and geometric decay.

#### Pedagogical prerequisites

CFT-33-001 and CFT-33-002; no Jin-route theorem is a prerequisite.

#### Lean correspondence

**EVIDENCE — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-003|CFT-CL-003]].**
The exact public navigation reexport
`CrouzeixTextbook.Part06.equation_three_lower_bound` points to the provider
[[formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean|DilationData.equation_three_lower_bound]],
and is declared in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]].
The provider applies
[[formalization/lean/Crouzeix/LoristSchwenninger/RecurrenceScalar.lean|recurrenceScalar_abs_le]]
to supply `M`, then uses the scalar declarations
[[formalization/lean/Crouzeix/LoristSchwenninger/Recurrence.lean|recurrence_finite_iteration]],
[[formalization/lean/Crouzeix/LoristSchwenninger/Recurrence.lean|bounded_recurrence_terminal_tendsto_zero]],
[[formalization/lean/Crouzeix/LoristSchwenninger/Recurrence.lean|inverse_power_weight_sum_tendsto]],
and
[[formalization/lean/Crouzeix/LoristSchwenninger/Recurrence.lean|recurrence_lower_bound]].
The compiled exercise theorem in the textbook module proves only the finite
scalar telescoping equality by induction, independently of Equation (3).

**Receipt audit.** The public declaration is at
`formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:46:5`; the provider
is at
`formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean:95`.
The formal mode is `reexported-proof`. Direct kernel dependency:
`CrouzeixConjecture.LoristSchwenninger.DilationData.equation_three_lower_bound`.

The fresh receipt records normalized type fingerprint
`610c49c8fbc4c14e142b48abb5085fee62003bb59d77058c588e3187321a897d`,
reported axioms `Classical.choice`, `Quot.sound`, and `propext`, and verification
target `CrouzeixTextbook`.

#### Historical context

**SOURCE CLAIM — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-006|CFT-CL-006]].**
The source recurrence and Equation (3) are pinned to
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L74-L90`.
The expanded denominator audit, finite endpoint display, scalar check, and ML
analogy here are Harp textbook exposition, not quoted source prose.

#### ML analogy

**Mathematical object / ML counterpart.** The inverse-power weighted scalar
recurrence is compared with a stability certificate for one fixed recurrent
linearization whose residual sequence has a deterministic uniform bound.

**Exact transfer.** The exact transfer is that reciprocal-gain weighting
converts local drift into a finite telescoping certificate: each weighted
adjacent term cancels before any limit is taken.

**Non-transfer.** The non-transfer boundary is that stochastic, nonlinear, or
time-varying recurrences require additional pathwise or probabilistic bounds;
the deterministic uniform bound used above does not transfer automatically.

**Diagnostic.** The diagnostic is to compute both the weighted terminal term
and the finite weighted residual sum, and reject the analogy if the terminal
term does not decay, because then the finite certificate does not justify the
claimed limiting inequality.

### CFT-33-004 — scalar combined inequality {#cft-33-004}

#### Purpose

**Motivation.** Turn the Hilbert-space displacement into a scalar energy
estimate, then place that estimate and the recurrence lower bound in one
inequality. The shared first moment is the elimination variable that makes the
sign contradiction possible.

#### Statement

For unit `x` with `κ=‖T‖` and `T*Tx=κ²x`, put
`b=‖displacement κ x‖²` and `m = Re ⟪x,E_1Tx⟫`. Source Equation (4) is

```text
b ≤ 2κ² - κm - κ³.
```

Combining it with source Equation (3) gives
`b(1 - 1/(κ-1)²) ≤ 2κ² - κ³`.

#### Hypothesis ledger

For Equation (4), `x` is a unit vector, `κ=‖T‖`, and `T*Tx=κ²x`. The
dilation map `V` is an isometry and `Q` is contractive. Norm attainment is
consumed upstream only to derive the singular-vector relation; it is not a
further input to Equation (4). For the scalar combination, `κ > 0`,
`(κ-1)² > 0`, Equation (3), and Equation (4) are the complete inputs. No sign
assumption on `b` is needed for CFT-33-004; `b≥0` enters only in CFT-33-005.

#### Proof roadmap

First expand the displacement norm and identify its cross term using the
`n=1` perturbation identity. Then substitute the lower bound for the first
perturbation moment into Equation (4), multiply only by positive quantities,
and collect the coefficient of `b`.

#### Proof

Define the dilation-space vector

```text
A = Q*VTx.
```

Then `d=A-κVx` and `b=‖d‖²`, so `b = ‖A-κVx‖²`. The complex Hilbert-space
norm identity, together with real `κ`, gives

```text
b = ‖A-κVx‖²
  = ‖A‖² - 2κ Re ⟪A,Vx⟫ + κ²‖Vx‖²
  = ‖A‖² - 2κ Re ⟪A,Vx⟫ + κ².
```

The last equality uses `‖Vx‖=‖x‖=1`. Here is the complete audit of the three
special properties of `x`.

**Unit norm.** The choice `‖x‖=1`, followed by the isometry identity
`‖Vx‖=‖x‖`, turns the last square into `κ²`.

**Norm attainment.** Norm attainment is consumed only when deriving
`T*Tx=κ²x`; it is not an additional input to Equation (4). The norm estimate
in Equation (4) instead uses contractivity of `Q*`, isometry of `V`, the
operator-norm inequality `T.le_opNorm`, `‖x‖=1`, and `κ=‖T‖`:

```text
‖A‖ ≤ ‖VTx‖ = ‖Tx‖ ≤ ‖T‖‖x‖ = κ,
```

and hence `‖A‖²≤κ²`. This is the same chain used by the provider theorem
`displacementSq_le` and by the CFT-33-006 assembly.

**Singular-vector equation.** Specializing the perturbation identity
`E_n=2V*(Q*)^nV-(T*)^n` to `n=1` and applying it to `Tx` gives

```text
E_1Tx = 2V*Q*VTx - T*Tx = 2V*A-T*Tx.
```

The order of the inner-product arguments matters here. In the starting moment
`m=Re ⟪x,E₁Tx⟫`, the term `V*A` begins inside the second argument because it
is a summand of `E₁Tx`. Real-part conjugate symmetry moves the entire `E₁Tx`
to the first argument: although conjugate symmetry conjugates the complex
inner product, taking the real part removes that conjugation. Substitute the
displayed perturbation identity there. Only then does adjoint transfer rewrite
`Re ⟪V*A,x⟫` as `Re ⟪A,Vx⟫`. The complete ordered calculation is

```text
Re ⟪x,E₁Tx⟫
  = Re ⟪E₁Tx,x⟫
  = 2 Re ⟪V*A,x⟫ - Re ⟪T*Tx,x⟫
  = 2 Re ⟪A,Vx⟫ - κ².
```

The final equality uses `T*Tx=κ²x` and `‖x‖=1`, since
`Re ⟪T*Tx,x⟫ = κ² Re ⟪x,x⟫ = κ²‖x‖² = κ²`. Thus, with
`m=Re ⟪x,E₁Tx⟫`, we have `m = 2 Re ⟪A,Vx⟫ - κ²`.

Consequently,

```text
b = ‖A‖² - 2κ Re ⟪A,Vx⟫ + κ²
  ≤ κ² - 2κ Re ⟪A,Vx⟫ + κ²
  = 2κ² - κ(m+κ²)
  = 2κ² - κm - κ³.
```

This is source Equation (4), pinned at
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98`.

We now combine it with Equation (3). That equation says

```text
-b/[κ(κ-1)²] ≤ m.
```

Because `κ>0`, multiplying by `κ` preserves the inequality. Negating reverses
it, and cancellation of the nonzero factor `κ` yields

```text
-κm ≤ b/(κ-1)².
```

Substitution in Equation (4), followed only by additive rearrangement, gives

```text
b ≤ 2κ² - κm - κ³
  ≤ 2κ² - κ³ + b/(κ-1)²,

b - b/(κ-1)² ≤ 2κ² - κ³,

b(1 - 1/(κ-1)²) ≤ 2κ² - κ³.
```

Notice what did *not* happen: we never multiplied the whole inequality by
`b`, so this combination is valid without knowing the sign of `b`.

**Worked instance.** At the sharp scalar boundary `κ=2`, choose `b=0` and
`m=0`. Equation (3), Equation (4), and the combined inequality all reduce to
`0≤0`. This checks why the argument cannot and should not exclude equality at
two.

#### Boundary case

At `κ=1`, the reciprocal coefficient is undefined. The eventual operator
assembly handles `κ≤1` before invoking this card. For `κ>1`, both `κ` and
`(κ-1)²` are strictly positive, so every division and cancellation above is
legal.

#### Pedagogical prerequisites

CFT-33-003.

#### Lean correspondence

The public theorem
`CrouzeixTextbook.Part06.scalar_combined_inequality` is declared in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]].
Its real hypotheses `hkpos`, `hkminus1sqpos`, `hlower`, and `hupper` correspond
respectively to positive `κ`, positive denominator square, Equation (3), and
Equation (4). It reexports the substantive scalar proof
[[formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean|scalar_combined_inequality_of_recurrence_bounds]].
The operator derivation of Equation (4) is compiled separately as
[[formalization/lean/Crouzeix/LoristSchwenninger/DisplacementUpper.lean|DilationData.displacementSq_le]];
its `hx`, `hκ`, and `hsingular` hypotheses are precisely the three audited
inputs above. Both are checked by `mise run lean-crouzeix-ls`; the public
wrapper is also checked by `mise run lean-crouzeix-textbook`.

**Receipt audit.** The public declaration is at
`formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:49:9`; the provider
is at `formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean:26`.
The formal mode is `reexported-proof`. Direct kernel dependency:
`CrouzeixConjecture.LoristSchwenninger.scalar_combined_inequality_of_recurrence_bounds`.

The fresh compiler receipt classifies the public wrapper as a `direct-alias`,
which is why the contract mode is truthfully `reexported-proof` rather than
`proved-here`. Its normalized type fingerprint is
`eb6c1ea9a5220026fc4b0cac90af21419688775723e9353193441e77bee69cad`;
the reported axioms are `Classical.choice`, `Quot.sound`, and `propext`, and
the verification target is `CrouzeixTextbook`.

#### Historical context

**SOURCE CLAIM — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-006|CFT-CL-006]].**
Equation (4) and its combination with Equation (3) are pinned to the exact
locator `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98`.

**EVIDENCE — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-003|CFT-CL-003]].**
Harp's local compiler receipt checks the named scalar theorem and provider
under their recorded hypotheses. Local compilation does not establish peer
review, acceptance, or journal publication, and the source attribution does
not establish historical priority.

#### ML analogy

**Mathematical object / ML counterpart.** The two bounds on one exact first
moment are compared with eliminating a shared coupling statistic from two
deterministic linear-stability certificates.

**Exact transfer.** In deterministic linear stability work, the exact scalar
pattern transfers whenever one certificate lower-bounds a coupling moment and
an independent energy expansion upper-bounds a nonnegative residual using the
same moment. Eliminating the shared moment is ordinary inequality algebra.

**Non-transfer.** The operator identities producing the two certificates do
not automatically transfer to a stochastic minibatch Jacobian, a nonlinear
state update, or a time-varying linearization: those settings need pathwise or
probabilistic replacements for norm attainment, adjoint identities, and the
uniform recurrence bound.

**Diagnostic.** Record both sides of the two input inequalities and the
combined residual. If the moment appearing in the lower certificate is not
the identical cross term in the energy expansion, reject the analogy rather
than eliminating two merely similar quantities.

### CFT-33-005 — scalar endpoint two {#cft-33-005}

#### Purpose

**Motivation.** Extract the sign contradiction that rules out `κ > 2`. All
operator theory has now been compressed into an ordered-field argument, so
this card identifies the exact point where nonnegativity is consumed.

#### Statement

For real `κ,b,m`, if `0≤b`, `0<κ`, `0<(κ-1)²`,
`-b/[κ(κ-1)²]≤m`, and `b≤2κ²-κm-κ³`, then `κ≤2`.

#### Hypothesis ledger

Nonnegativity `b ≥ 0`, positivity of `κ` and `(κ-1)²`, and the lower and upper
moment inequalities entering CFT-33-004.

#### Proof roadmap

Assume `κ > 2`: then `1 - 1/(κ-1)² > 0`, while
`2κ²-κ³ = κ²(2-κ) < 0`, contradicting `b ≥ 0`.

#### Proof

The conclusion `κ≤2` is immediate in two regimes and contradictory in the
third.

- If `κ≤1`, then certainly `κ≤2`; the recurrence denominator is never used.
- If `1<κ≤2`, the conclusion already holds. Within this regime, `1<κ<2`
  gives the desired strict inequality, while the endpoint is handled next.
- If `κ=2`, the conclusion is equality. The combined inequality has right
  side `2κ²-κ³=0`, so no strict contradiction should be expected.

It remains to exclude `κ>2`. Then `κ-1>1`, so `(κ-1)²>1`. Division by the
positive square gives `1/(κ-1)²<1`, hence

```text
0<1-1/(κ-1)².
```

Because `b=‖d‖²`, we have `b≥0`; multiplication by the positive factor gives

```text
0≤b(1-1/(κ-1)²).
```

But `κ>2` also implies `κ²>0` and `2-κ<0`, whence

```text
2κ²-κ³ = κ²(2-κ)<0.
```

CFT-33-004 would put the nonnegative left side below this negative right side,
an impossibility. Thus `κ>2` is false, and `κ≤2`.

**Worked instance.** If `κ=3`, the left coefficient is
`1-1/(3-1)²=3/4`, so `b≥0` makes the left side nonnegative. The right side is
`2·3²-3³=-9`. Thus the combined inequality would demand a nonnegative number
be at most `-9`, displaying the contradiction numerically without replacing
the symbolic proof.

#### Boundary case

At `κ=2`, the coefficient on the left is also zero:
`1-1/(2-1)²=0`. The inequality reads `0≤0`, exactly matching the sharp
endpoint rather than ruling it out.

#### Pedagogical prerequisites

CFT-27-004 and CFT-33-004.

#### Lean correspondence

The public theorem `CrouzeixTextbook.Part06.scalar_endpoint_two` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]]
maps `hb` to `b≥0`, passes the four CFT-33-004 hypotheses unchanged, and
returns `κ≤2`. It reexports
[[formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean|scalar_endpoint_le_two]],
whose inner helper `scalar_contradiction_le_two` contains the positive-factor
and negative-right-side proof. The public and provider declarations are built
by `lean-crouzeix-textbook` and `lean-crouzeix-ls`, respectively.

**Receipt audit.** The public declaration is at
`formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:60:9`; the provider
is at `formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean:78`.
The formal mode is `reexported-proof`. Direct kernel dependency:
`CrouzeixConjecture.LoristSchwenninger.scalar_endpoint_le_two`.

The fresh compiler receipt classifies this public wrapper as a `direct-alias`
and the contract as `reexported-proof`. Its normalized type fingerprint is
`d0d9e72ee420d466774132edb0b84345a1882462e1e68c37702786c94baaf02c`;
the reported axioms are `Classical.choice`, `Quot.sound`, and `propext`, and
the verification target is `CrouzeixTextbook`.

#### Historical context

**SOURCE CLAIM — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-006|CFT-CL-006]].**
The scalar endpoint is pinned to the exact locator
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98`.

**EVIDENCE — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-003|CFT-CL-003]].**
Harp's local compiler receipt checks the scalar endpoint theorem under its
recorded hypotheses. Local compilation does not establish peer review,
acceptance, or journal publication, and the source attribution does not
establish historical priority.

#### ML analogy

**Mathematical object / ML counterpart.** The ordered-field sign
contradiction is compared with a certified nonnegative energy and a certified
strictly negative upper bound in a stability audit.

**Exact transfer.** A nonnegative empirical energy cannot be at most a
strictly negative certified upper bound; this sign contradiction is exact and
does not depend on operator theory once the combined scalar inequality exists.

**Non-transfer.** An estimated loss, Monte Carlo confidence interval, or
floating-point residual is not literally nonnegative-and-bounded in the same
ordered field unless its error bars are included. Sampling or rounding can
therefore destroy the strict sign gap.

**Diagnostic.** Compute the factor `1-1/(κ-1)²` and the margin
`κ²(2-κ)` separately. Only report a contradiction when the former has a
certified positive lower bound and the latter a certified negative upper
bound.

### CFT-33-006 — perturbation lemma {#cft-33-006}

#### Purpose

**Motivation.** Assemble the whole operator-level argument: reduce the norm problem to one
unit top singular vector, turn the two operator estimates into a scalar
certificate, and close that certificate at the constant two.

#### Statement

For source-faithful finite-dimensional `DilationData`, `‖T‖ ≤ 2`.

#### Hypothesis ledger

Every hypothesis is listed here with the first place at which it is consumed.

**Finite dimensionality.** `[FiniteDimensional ℂ E]` is consumed by
`exists_unit_norm_attaining_and_adjoint_apply`: the unit sphere is compact, so
the supremum in the operator norm is attained. It is also an explicit
parameter of `DilationData`.

**Completeness of `E`.** `[CompleteSpace E]` is consumed when Hilbert adjoints
are formed and in the Rayleigh-quotient argument that converts norm attainment
into the singular-vector equation.

**Nontriviality.** `[Nontrivial E]` is consumed by norm attainment to ensure
that the unit sphere is nonempty; it also supports the norm-one statement for
the isometric map `V` used in the displacement estimates.

**Completeness of `K`.** `[CompleteSpace K]` is consumed when the adjoints
`V*` and `Q*` are formed on the dilation space.

**Isometry of `V`.** `V_isometry` is consumed by `V_apply_norm` and by the
contractivity of `V*`; these enter the adjacent-defect bound in Equation (3)
and the estimate `‖VTx‖=‖Tx‖` in Equation (4).

**Contractivity of `Q`.** `Q_norm_le_one` is consumed to show that every
`(Q*)^n` is contractive in Equation (3), and that
`‖Q*VTx‖≤‖VTx‖` in Equation (4).

**Exact perturbation identity.** `perturbation_eq` is consumed twice: first to
write `E_n=2V*(Q*)^nV-(T*)^n`, which supplies the uniformly bounded power
identity and the recurrence behind Equation (3), and then at `n=1` to expand
the cross term in Equation (4).

**Uniform perturbation bounds.** `bound_nonneg` and
`perturbation_norm_le` are consumed by `recurrenceScalar_abs_le`; together
with the power bound they make the terminal term in the inverse-power
telescoping argument tend to zero.

**Commutation and power identities.** `commutes_with_target` is consumed by
the scalar bridge `E_nT^n=T^nE_n` and the adjacent step
`TE_{n+1}x=E_{n+1}Tx`. Ordinary adjoint-power identities are then consumed to
propagate `T*Tx=κ²x` to `(T*)^(n+1)Tx=κ²(T*)^nx`.

**Norm attainment.** Finite dimensionality and nontriviality produce `x` with
`‖x‖=1` and `‖Tx‖=κ`. The unit-norm equality is consumed by both Equations
(3) and (4). The norm-attainment equality is consumed to derive the next
singular-vector relation; it is not an additional input to Equation (4).

**Singular-vector relation.** `T*T x=κ²x` is consumed by the exact recurrence
in Equation (3) and by the cross-term calculation in Equation (4).

#### Proof roadmap

Handle `‖T‖ ≤ 1` directly; otherwise choose a unit norm-attaining vector,
instantiate Equations (3) and (4), and apply CFT-33-005.

#### Proof

Write `κ = ‖T‖` and split exactly as the provider does:

```text
by_cases hsmall : κ ≤ 1.
```

In the small branch, `κ ≤ 1 < 2`, so transitivity gives `κ≤2`; this branch
never constructs a vector and never divides by `κ-1`.

In the other branch, negating `κ≤1` gives `1 < κ`. Apply
`exists_unit_norm_attaining_and_adjoint_apply T`. Finite-dimensional norm
attainment and the Rayleigh-quotient lemma return a vector `x` with

```text
‖x‖ = 1,    ‖Tx‖ = κ,    T*T x = κ²x.
```

The provider names the middle equality `_hTx`, because the helper has already
consumed it to prove the singular-vector relation. Norm attainment is consumed
upstream to produce `T*T x=κ²x`; it is not an additional input to Equation
(4). Thus no hypothesis silently disappears: the derived singular relation
supersedes the premise from which it was obtained.

Now use precisely the scalar variables of CFT-33-005:

```text
b = ‖Q*VTx-κVx‖²,    m = Re ⟪E₁Tx,x⟫.
```

Equation (3), namely `equation_three_lower_bound`, is instantiated with
`1<κ`, `‖x‖=1`, and `T*T x=κ²x`. Its conclusion initially places `x` in the
first inner-product slot. Real-part conjugate symmetry, implemented by
`inner_re_symm`, changes only the presentation and yields

```text
-b/[κ(κ-1)²] ≤ m.
```

Equation (4), namely `displacementSq_le`, is instantiated with `‖x‖=1`, the
reflexive identification `κ=‖T‖`, and the same singular-vector relation. It
does not replace an operator-norm inequality by the norm-attainment equality.
Writing `A=Q*VTx`, its provider-faithful chain is

```text
‖A‖ ≤ ‖Tx‖ ≤ ‖T‖‖x‖ = κ.
```

The first displayed step abbreviates
`‖A‖≤‖VTx‖=‖Tx‖`: contractivity of `Q*` followed by isometry of `V`.
The next step is `T.le_opNorm`, and the last uses `‖x‖=1` and `κ=‖T‖`.
Together with the singular relation for the cross term, this yields

```text
b ≤ 2κ²-κm-κ³.
```

The remaining inputs to the scalar endpoint are explicit. The definition of
`b` gives `0 ≤ b`; `1<κ` gives `0 < κ`; and subtracting one and squaring gives
`0 < (κ-1)²`. Therefore `scalar_endpoint_le_two` consumes, in order,

```text
0 ≤ b,
0 < κ,
0 < (κ-1)²,
-b/[κ(κ-1)²] ≤ m,
b ≤ 2κ²-κm-κ³,
```

and returns `κ ≤ 2`. Since `κ=‖T‖`, this is the target `‖T‖≤2`.

**Worked instance.** If `‖T‖=1/2`, the small branch gives
`‖T‖=1/2≤1≤2` and constructs no singular vector. This concrete value checks
that the proof's denominator-bearing machinery is confined to the branch
where `1<‖T‖`.

#### Boundary case

The small-norm branch avoids every division by `κ-1`.

#### Pedagogical prerequisites

CFT-23-002, CFT-23-005, and CFT-33-005.

#### Lean correspondence

The exact public declaration `CrouzeixTextbook.Part06.perturbation_lemma` is in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]].
It reexports the complete provider proof
`CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two` in
[[formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean|PerturbationLemma.lean]].
The public hypothesis map is identity-on-data: its finite-dimensional Hilbert
spaces and `DilationData` argument are passed unchanged, and its conclusion is
the same `‖data.T‖≤2`.

**Receipt audit.** The public declaration is at
`formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:72:5`; the provider
is at
`formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean:27`.
The formal mode is `reexported-proof`. Direct kernel dependency:
`CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two`.

The fresh compiler receipt classifies this as `reexported-proof`. Its
normalized type fingerprint is
`7d123a11205755170fe7ae624c912e72ef2840c6b6f0b1add63c3a4ed123a0fc`;
the reported axioms are `Classical.choice`, `Quot.sound`, and `propext`, and
the verification target is `CrouzeixTextbook`. The provider is independently
built by `lean-crouzeix-ls`.

#### Historical context

**EVIDENCE — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-003|CFT-CL-003]].**
The locally compiled provider is
[[formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean|PerturbationLemma.lean]];
its reproduction command and verification boundary are recorded in the claim
ledger.

**SOURCE CLAIM — [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-006|CFT-CL-006]].**
The captured source presents the perturbation lemma at
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99`. That locator supports
source attribution, not local compilation status or peer-review status.

**No priority claim.** These labels report what the pinned source says and
what the local compiler checks. They do not assert who first discovered the
argument.

#### ML analogy

**Mathematical object / ML counterpart.** The theorem concerns one fixed
deterministic operator `T` together with concrete dilation data `V,Q,E₁,...`.
The bounded ML analogy is a recurrent-linearization stability audit for one
frozen Jacobian plus explicit auxiliary maps playing the roles of `V`, `Q`,
and `E₁`. Those data, not `T` or the Jacobian alone, define
`b=‖Q*VTx-κVx‖²` and `m=Re⟪E₁Tx,x⟫`.

**Exact transfer.** Once a model-specific argument supplies real scalars
`κ,b,m` with the same scalar certificate `0≤b`, `0<κ`, `0<(κ-1)²`,
`-b/[κ(κ-1)²]≤m`, and `b≤2κ²-κm-κ³`, the conclusion `κ≤2` transfers
exactly. No operator analogy is needed in this last scalar step.

**Non-transfer.** A Jacobian matrix by itself supplies the singular-value
calculation, but it does not supply the dilation certificate. For the fixed
matrix below, `J` alone does not determine `b` or `m`, which require concrete
dilation data `V,Q,E₁`. Stochastic or time-varying Jacobians additionally
require pathwise or probabilistic bounds controlling drift, noise, and the
terminal term.

**Diagnostic.** Take the fixed nonnormal real Jacobian
`J = [[1,4],[0,1]]`. Here `J*` is the transpose (the conjugate transpose over
complex scalars). Direct multiplication gives

```text
J*J = [[1,4],[4,17]].
```

Its characteristic polynomial is `λ²-18λ+1`, so its largest eigenvalue is
`9+4√5=(2+√5)²`. Hence

```text
κ = 2+√5 = ‖J‖,
x = (1,κ)/√(1+κ²).
```

Because `κ²=1+4κ`, direct multiplication before normalization gives

```text
[[1,4],[4,17]] [1,κ]ᵀ
  = [1+4κ,4+17κ]ᵀ
  = [κ²,κ³]ᵀ
  = κ²[1,κ]ᵀ.
```

Therefore `J*Jx=κ²x`; the singular residual is exactly zero. This computable
fact does not complete the LS certificate. Moreover `κ>2`, so the five scalar
inequalities are infeasible for every `b≥0` and every `m`: if they held
together, the scalar endpoint would force `κ≤2`. Reject the scalar certificate
unless the auxiliary data are supplied and every operator hypothesis is
checked.

## Worked examples

**Example 1.** A toy recurrence `m_{n+1} ≤ κm_n-c` with bounded `m_n`
cannot persist for `κ>1` when `c` has the wrong sign after inverse-power
summation. The actual proof engineers `c` from the displacement norm.

**Example 2.** Finite dimensionality supplies a norm-attaining unit vector for
`T`. In an arbitrary Banach space the supremum defining `‖T‖` need not be
attained, so an approximation argument or stronger hypothesis would be needed.

## ML bridge

The mechanism resembles a stability contradiction for a recurrent
linearization: a uniform state bound conflicts with a one-step drift that
accumulates geometrically. This is only an analogy. The theorem assumes one
fixed operator, exact commutation, and deterministic uniform bounds; it does
not transfer unchanged to stochastic or time-varying Jacobians.

## Lean translation

The six public names in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]]
compile against the substantive maintained provider. CFT-33-001 through
CFT-33-006 have exact correspondence and reconstructible prose. Exercises
E01--E06 have separate compiled solution theorems and source starters. The
chapter counters below project those six contract rows from a fresh compiler
receipt.

The durable compiler receipt recorded in the tracked
[[knowledge/crouzeix_textbook/status_and_scope|status and scope]] surface has:

- Compiler receipt SHA-256:
  `c7c29bc972ef9519067fc3e2d7e365d3601997f204d84052bedcf0d300b8fd49`;
- serialized bytes: `527908`; and
- declaration count: `447`.

The compiler receipt and the six-ledger publication generation are distinct
identities. The receipt digest hashes the exact serialized compiler output;
the publication generation hashes the six rendered ledger files. Neither
identity certifies the historical or ML prose.

## Exercises

### CFT-33-E01 -- retrieval {#exercise-cft-33-e01}

Prove the successor singular-vector bridge
`(T*)^(n+1)Tx=κ²(T*)^n x`. Lean starter: expose the successor power with
`rw [pow_succ, mul_apply_eq_comp]`, then use `hsingular` and linearity. The
compiled solution is
`CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_01_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]].

### CFT-33-E02 -- calculation {#exercise-cft-33-e02}

For `a>0`, prove
`-‖z‖²/(4a) ≤ a‖u‖²-Re ⟪z,u⟫` by completing the square. Lean starter:
instantiate the exact completed-square identity and isolate the nonnegative
term. The distinct compiled solution is
`CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_02_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]].

### CFT-33-E03 -- written-proof {#exercise-cft-33-e03}

For `κ : ℝ`, `m : ℕ → ℝ`, and `N : ℕ`, prove the exact finite identity

```text
∑ i∈Finset.range N [(κ⁻¹)^(i+1)m_(i+1) - (κ⁻¹)^(i+2)m_(i+2)]
= κ⁻¹m_1 - (κ⁻¹)^(N+1)m_(N+1).
```

Before induction, expand the first few terms (`i=0,1,2`) and mark each
positive/negative cancellation and the two surviving endpoints. Then induct on
`N`, expose `Finset.sum_range_succ`, and normalize the endpoint terms. The
compiled solution is
`CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_03_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]].

### CFT-33-E04 -- written-proof {#exercise-cft-33-e04}

Let `κ : ℝ` and assume `1<κ`. Prove the denominator/sign audit

```text
0<κ,    0<(κ-1)²,    and    κ(κ-1)²≠0.
```

Do not ask an automation tactic to clear the denominator before establishing
these facts. Lean starter: name `0<κ` and `0<κ-1`, obtain square positivity,
then use `mul_ne_zero`. The distinct compiled solution is
`CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_04_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]].

### CFT-33-E05 -- boundary {#exercise-cft-33-e05}

For real `κ,b`, assume

```text
0≤b
```

and

```text
b(1-1/(κ-1)²)≤2κ²-κ³.
```

Prove `κ≤2`. Your proof must assume `2<κ`, show the left side is nonnegative,
factor the right side as `κ²(2-κ)`, and derive the sign contradiction. Lean
starter: use `by_contra`, prove `hfactor_pos` and `hright_neg` separately, and
connect them through the combined inequality. The distinct compiled solution
is `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_05_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]].

For auditability, both exercise solutions compile as genuine `theorem`
declarations rather than aliases. E04 maps `hκ` to `1<κ` and returns the two
positivity facts plus the nonzero product; its normalized type fingerprint is
`9f496c7d5a22e792008c61c6f53c6191cca2373e96b43bab5892bb6399b3f99c`.
E05 maps `hb` to `0≤b` and `hcombined` to the displayed combined inequality;
its normalized type fingerprint is
`71c430c3b0eae949f33b2e2f7b324dd83046d9e27f39348a2b4fd336e18ae3da`.
Both receipts report `Classical.choice`, `Quot.sound`, and `propext` under the
`CrouzeixTextbook` verification target. Their normalized statements differ
from every public Chapter 33 checkpoint, and their proof bodies do not invoke
`scalar_combined_inequality`, `scalar_contradiction_le_two`, or
`scalar_endpoint_two`.

### CFT-33-E06 -- lean-proof {#exercise-cft-33-e06}

Let `data` be source-faithful `DilationData`, let `κ : ℝ`, and assume
`κ = ‖T‖`. Your task is to prove `‖T‖≤2` by carrying out the assembly rather
than receiving its scalar certificate. Split on `κ≤1`. In the other branch
derive `1<κ`, obtain a norm-attaining unit vector `x`, and use norm attainment
to obtain `T*T x=κ²x`. Define

```text
b=‖Q*VTx-κVx‖²,    m=Re⟪E₁Tx,x⟫.
```

Invoke Equation (3), use real-part conjugate symmetry, derive Equation (4),
prove the three sign conditions, and only then call the scalar endpoint. The
compiled solution is
`CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_06_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]].
Its statement differs from every public Chapter 33 checkpoint, and its proof
does not invoke `perturbation_lemma`, `norm_target_le_two`, or
`scalar_endpoint_two`.

The Lean type map sends `data` to the source-faithful dilation package, `κ` to
the named operator norm, and `hκ` to the identification `κ=‖data.T‖`. There
are no input hypotheses for `x`, `b`, or `m`: the proof obtains the vector and
derives both scalar inequalities. Its normalized type fingerprint is
`6d8a70779a3993e87edea9466272c8742e362b2ecf9b72431465fd03ea7361c9`.
The receipt reports direct dependencies on norm attainment, Equation (3),
Equation (4), displacement nonnegativity, and the scalar endpoint; its axioms
are `Classical.choice`, `Quot.sound`, and `propext` under verification target
`CrouzeixTextbook`.

### Exercise Lean audit

The starter and solution positions below come from the same compiler receipt.
All positions are in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]],
all six solution declarations are compiler-classified `theorem`s, and every
normalized statement differs from all six public Chapter 33 statements.

| Exercise | Declaration | Starter / solution location | Type SHA-256 |
|---|---|---|---|
| CFT-33-E01 | `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_01_solution` | `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:82:1` / `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:83:9` | `19bf70d60d694bdc94bc1c7c86b28075b96157a35006bed74fc958f0b3d89a9c` |
| CFT-33-E02 | `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_02_solution` | `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:95:1` / `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:96:9` | `e04f6c807864c88c2c8cdcb2a4aed4a58135bbaa0efe59e92a672b745ca9980f` |
| CFT-33-E03 | `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_03_solution` | `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:103:1` / `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:105:9` | `5dff70645a37f76f6d11a4fc3d33ab9457ecac473765d61f60772b4fdd2fe736` |
| CFT-33-E04 | `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_04_solution` | `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:117:1` / `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:119:9` | `9f496c7d5a22e792008c61c6f53c6191cca2373e96b43bab5892bb6399b3f99c` |
| CFT-33-E05 | `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_05_solution` | `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:131:1` / `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:133:9` | `71c430c3b0eae949f33b2e2f7b324dd83046d9e27f39348a2b4fd336e18ae3da` |
| CFT-33-E06 | `CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_06_solution` | `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:154:1` / `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean:157:9` | `6d8a70779a3993e87edea9466272c8742e362b2ecf9b72431465fd03ea7361c9` |

**Direct kernel dependencies.** E01 reports `DilationData` and
`DilationData.T`; E02 reports `pre_square_lower_bound`; E03, E04, and E05
report no maintained direct declaration. E06 reports the exact assembly set:
`DilationData`, `DilationData.T`, `DilationData.displacement`,
`DilationData.displacementSq`, `DilationData.displacementSq.eq_1`,
`DilationData.displacementSq_le`, `DilationData.displacementSq_nonneg`,
`DilationData.equation_three_lower_bound`,
`DilationData.firstPerturbationMoment`,
`DilationData.firstPerturbationMoment.eq_1`, `DilationData.perturbation`,
`exists_unit_norm_attaining_and_adjoint_apply`,
`finiteDimensionalCompleteSpace`, and `scalar_endpoint_le_two`, all in the
`CrouzeixConjecture.LoristSchwenninger` namespace.

All six exercise receipts report axioms `Classical.choice`, `Quot.sound`, and
`propext`, with verification target `CrouzeixTextbook`. Their locations and
dependencies belong to the compiler receipt identity stated in the Lean
translation section above, not to the separate six-ledger generation digest.

### Solution sketches

These are complete mathematical solutions; the linked Lean theorems above
check the corresponding formal statements.

#### CFT-33-E01 solution

Write `A=T*`. The successor law `pow_succ` gives

```text
A^(n+1)(Tx)=A^n(A(Tx)).
```

The singular-vector hypothesis replaces the inner term by `κ²x`. Since
`A^n` is linear,

```text
A^n(κ²x)=κ²A^n x.
```

Combining the two equalities yields
`(T*)^(n+1)Tx=κ²(T*)^n x`. In Lean, `mul_apply_eq_comp` exposes the operator
product, `hsingular` performs the central rewrite, and `map_smul` moves the
scalar through the linear map.

#### CFT-33-E02 solution

Put `w=u-z/(2a)`. Because `a>0`, the quantity `a‖w‖²` is nonnegative. The
completed-square identity proved in the card is

```text
a‖w‖²-‖z‖²/(4a)=a‖u‖²-Re⟪z,u⟫.
```

Subtracting `‖z‖²/(4a)` from `0≤a‖w‖²` gives

```text
-‖z‖²/(4a) ≤ a‖w‖²-‖z‖²/(4a)
             = a‖u‖²-Re⟪z,u⟫.
```

Thus the claim follows by discarding exactly one nonnegative square. The Lean
solution instantiates `pre_square_lower_bound`; its proof has precisely this
discarded square as the slack term.

#### CFT-33-E03 solution

Set `q=κ⁻¹`. For `N=0`, the sum is empty and the right side is
`qm_1-qm_1=0`. Assume the formula holds at `N`. The identity
`Finset.sum_range_succ` adds the term with index `N`, so

```text
S_(N+1)
= [qm_1-q^(N+1)m_(N+1)]
  +[q^(N+1)m_(N+1)-q^(N+2)m_(N+2)]
= qm_1-q^(N+2)m_(N+2).
```

The middle endpoint cancels exactly. This is the required formula with
`N+1` in place of `N`, so induction proves it for every natural `N`. No
positivity assumption on `κ` is needed for this finite algebraic identity.

#### CFT-33-E04 solution

From `1<κ` and `0<1`, transitivity gives `0<κ`. Subtracting one gives
`0<κ-1`, and squaring a strictly positive real gives `0<(κ-1)²`. Strictly
positive reals are nonzero, so both factors `κ` and `(κ-1)²` are nonzero.
Applying `mul_ne_zero` yields

```text
κ(κ-1)²≠0.
```

These are exactly the facts that later justify division by the recurrence
denominator.

#### CFT-33-E05 solution

Suppose for contradiction that `2<κ`. Then `1<κ-1`, hence
`1<(κ-1)²`. Since the square is positive, division preserves order and gives
`1/(κ-1)²<1`; therefore `0<1-1/(κ-1)²`. With `0≤b`,

```text
0≤b(1-1/(κ-1)²).
```

On the other hand,

```text
2κ²-κ³=κ²(2-κ)<0,
```

because `κ²>0` and `2-κ<0`. The assumed combined inequality places the
nonnegative left side below this strictly negative value, a contradiction.
Thus `κ≤2`.

#### CFT-33-E06 solution

Split on `κ≤1`. In that branch, `‖T‖=κ≤1≤2`. Otherwise `1<κ`. Apply
`exists_unit_norm_attaining_and_adjoint_apply` to obtain `x` with `‖x‖=1`
and `T*Tx=κ²x`. Define

```text
b=displacementSq κ x,
m=firstPerturbationMoment x.
```

The theorem `equation_three_lower_bound`, followed by `inner_re_symm`, gives
`-b/[κ(κ-1)²]≤m`. The theorem `displacementSq_le` gives
`b≤2κ²-κm-κ³`. The definition of a squared norm gives `0≤b`; from `1<κ`
we get `0<κ` and `0<(κ-1)²`. Feed these five facts to
`scalar_endpoint_le_two` to obtain `κ≤2`. Finally rewrite
`κ=‖T‖`. This derivation constructs its singular vector and both scalar bounds;
it never invokes the already assembled perturbation lemma.

### Exercise contract snapshot

| Metric | Count |
| --- | ---: |
| solved exercises | 6 |
| unresolved exercises | 0 |

E01--E06 have registered starters and distinct compiled Lean solutions.

## Synthesis and forward dependencies

This chapter's LS chain depends only on the common Chapters 23, 27, and 29
machinery plus earlier CFT-33 nodes; it does not pass through Chapters 30--32's
Jin branch. CFT-33-006 hands the perturbation lemma to Chapter 34's realization
step. The compiler verifies the named provider theorems, and the six workshops
make the analytic and scalar assembly reconstructible without crossing into
the Jin branch.
