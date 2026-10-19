import CrouzeixConjecture.Definitions

/-!
Harp-owned formal surface for the Lorist-Schwenninger perturbation route.

The source route starts with a compressed contraction-power family and
perturbations

```
E n = 2 V^* Q^{*n} V - T^{*n}.
```

This module records the abstract interface and proves the first boundedness
contracts used before the scalar recurrence.  The recurrence itself remains a
separate source-mapped proof obligation.
-/

noncomputable section

namespace CrouzeixConjecture
namespace LoristSchwenninger

open scoped Matrix.Norms.L2Operator

variable {n : Type*} [Fintype n] [DecidableEq n]

/-- The finite-dimensional perturbation data in Lorist-Schwenninger's Lemma 1,
specialized to Harp's matrix model.  `compressionPower` is the compressed
contraction-power family `V^* Q^{*n} V`; this interface records exactly the
properties needed before the recurrence proof starts. -/
structure PerturbationFamily (T : SquareMatrix n) where
  compressionPower : ℕ → SquareMatrix n
  perturbation : ℕ → SquareMatrix n
  perturbation_eq :
    ∀ k, perturbation k = (2 : ℂ) • compressionPower k - (Matrix.conjTranspose T) ^ k
  compressionPower_norm_le_one : ∀ k, ‖compressionPower k‖ ≤ 1
  perturbation_norm_le : ∃ M : ℝ, 0 ≤ M ∧ ∀ k, ‖perturbation k‖ ≤ M
  commutes_with_target : ∀ k, Commute (perturbation k) T

/-- A named version of the hypothesis package for LS source-map registration. -/
def PerturbationLemmaHypotheses (T : SquareMatrix n) : Prop :=
  ∃ _data : PerturbationFamily T, True

/-- Uniform boundedness gives a reusable bound for any selected perturbation. -/
theorem perturbation_norm_le_bound
    {T : SquareMatrix n} (data : PerturbationFamily T) :
    ∃ M : ℝ, 0 ≤ M ∧ ∀ k, ‖data.perturbation k‖ ≤ M :=
  data.perturbation_norm_le

/-- The compressed contraction-power term has norm at most `2` after the
source's leading scalar factor. -/
theorem two_smul_compressionPower_norm_le_two
    {T : SquareMatrix n} (data : PerturbationFamily T) (k : ℕ) :
    ‖(2 : ℂ) • data.compressionPower k‖ ≤ 2 := by
  rw [norm_smul]
  have hscale : ‖(2 : ℂ)‖ = (2 : ℝ) := by norm_num
  rw [hscale]
  nlinarith [data.compressionPower_norm_le_one k,
    norm_nonneg (data.compressionPower k)]

/-- Equation 1 in the LS proof directly bounds all target adjoint powers. -/
theorem adjoint_power_norm_le_two_add_bound
    {T : SquareMatrix n} (data : PerturbationFamily T)
    {M : ℝ} (hM : ∀ k, ‖data.perturbation k‖ ≤ M) (k : ℕ) :
    ‖(Matrix.conjTranspose T) ^ k‖ ≤ 2 + M := by
  have hterm := two_smul_compressionPower_norm_le_two data k
  have hsum :
      ‖(Matrix.conjTranspose T) ^ k‖ ≤
        ‖(2 : ℂ) • data.compressionPower k‖ +
          ‖data.perturbation k‖ := by
    have hrew :
        (Matrix.conjTranspose T) ^ k =
          (2 : ℂ) • data.compressionPower k - data.perturbation k := by
      rw [data.perturbation_eq k]
      abel
    rw [hrew]
    exact norm_sub_le _ _
  exact hsum.trans (add_le_add hterm (hM k))

/-- The same bound, transported from adjoint powers to target powers. -/
theorem target_power_norm_le_two_add_bound
    {T : SquareMatrix n} (data : PerturbationFamily T)
    {M : ℝ} (hM : ∀ k, ‖data.perturbation k‖ ≤ M) (k : ℕ) :
    ‖T ^ k‖ ≤ 2 + M := by
  have hstar := adjoint_power_norm_le_two_add_bound data hM k
  have hnorm : ‖T ^ k‖ = ‖(Matrix.conjTranspose T) ^ k‖ := by
    rw [← Matrix.conjTranspose_pow, Matrix.l2_opNorm_conjTranspose]
  exact hnorm.trans_le hstar

/-- Product boundedness for the terminal term in the LS recurrence. -/
theorem perturbation_mul_target_power_norm_le
    {T : SquareMatrix n} (data : PerturbationFamily T)
    {M : ℝ} (hMnonneg : 0 ≤ M) (hM : ∀ k, ‖data.perturbation k‖ ≤ M)
    (k : ℕ) :
    ‖data.perturbation k * T ^ k‖ ≤ M * (2 + M) := by
  exact (norm_mul_le _ _).trans
    (mul_le_mul (hM k) (target_power_norm_le_two_add_bound data hM k)
      (norm_nonneg _) hMnonneg)

/-- The source's abstract perturbation lemma, represented as the remaining
route contract after the already-formalized terminal boundedness estimates. -/
def PerturbationLemmaStatement : Prop :=
  ∀ T : SquareMatrix n, PerturbationFamily T → ‖T‖ ≤ 2

/-- Source-map alias for the full LS perturbation lemma obligation. -/
def PerturbationLemma : Prop :=
  PerturbationLemmaStatement (n := n)

/-- The recurrence proof is the next open LS mathematical obligation. -/
def PowerRecurrenceStatement : Prop :=
  ∀ T : SquareMatrix n, PerturbationFamily T → ‖T‖ ≤ 2

/-- Source-map alias for the scalar recurrence obligation inside the LS proof. -/
def PowerRecurrence : Prop :=
  PowerRecurrenceStatement (n := n)

/-- Source-map alias for the terminal LS Crouzeix route obligation. -/
def CrouzeixTerminal : Prop :=
  PerturbationLemmaStatement (n := n)

/-- Once the abstract perturbation lemma is available, the finite-dimensional
constant-two endpoint follows for any LS perturbation family. -/
theorem norm_le_two_of_perturbationFamily
    (hLS : PerturbationLemmaStatement (n := n))
    {T : SquareMatrix n} (data : PerturbationFamily T) :
    ‖T‖ ≤ 2 :=
  hLS T data

end LoristSchwenninger
end CrouzeixConjecture
