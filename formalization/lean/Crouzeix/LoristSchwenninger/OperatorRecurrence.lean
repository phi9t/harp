import Crouzeix.LoristSchwenninger.CompletedSquare
import Crouzeix.LoristSchwenninger.Recurrence
import Crouzeix.LoristSchwenninger.RecurrenceScalar

/-!
The operator recurrence at the center of the Lorist--Schwenninger
perturbation lemma.
-/

noncomputable section

open scoped InnerProduct

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {E K : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace ℂ E]
  [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E]
variable [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]

namespace DilationData

omit [Nontrivial E] in
private theorem recurrenceScalar_eq_inner
    (data : DilationData (E := E) (K := K)) (n : ℕ) (x : E) :
    recurrenceScalar data x n =
      (inner ℂ (data.perturbation n x)
        ((ContinuousLinearMap.adjoint data.T ^ n) x)).re := by
  rw [recurrenceScalar, (data.commutes_with_target n).pow_right n |>.eq]
  simp only [mul_apply_eq_comp]
  rw [← (data.T ^ n).adjoint_inner_left (data.perturbation n x) x]
  rw [← ContinuousLinearMap.star_eq_adjoint (data.T ^ n), star_pow,
    ContinuousLinearMap.star_eq_adjoint data.T]
  exact @inner_re_symm ℂ E _ _ _
    ((ContinuousLinearMap.adjoint data.T ^ n) x)
    (data.perturbation n x)

omit [Nontrivial E] in
/-- The denominator-free operator identity preceding completion of the square. -/
theorem recurrence_difference_identity
    (data : DilationData (E := E) (K := K))
    {κ : ℝ} (n : ℕ) (x : E)
    (hsingular :
      ContinuousLinearMap.adjoint data.T (data.T x) =
        ((κ ^ 2 : ℝ) : ℂ) • x) :
    κ * recurrenceScalar data x n - recurrenceScalar data x (n + 1) =
      (κ ^ 2 - κ) * ‖(ContinuousLinearMap.adjoint data.T ^ n) x‖ ^ 2 -
        (inner ℂ (data.adjacentPowerDefect κ n x)
          ((ContinuousLinearMap.adjoint data.T ^ n) x)).re := by
  rw [data.recurrenceScalar_eq_inner n x,
    data.recurrenceScalar_eq_inner (n + 1) x]
  have hcomm :
      data.T (data.perturbation (n + 1) x) =
        data.perturbation (n + 1) (data.T x) := by
    have h := congrArg (fun A : E →L[ℂ] E => A x)
      (data.commutes_with_target (n + 1)).eq
    simpa [mul_apply_eq_comp] using h.symm
  have hsucc :
      (inner ℂ (data.perturbation (n + 1) x)
        ((ContinuousLinearMap.adjoint data.T ^ (n + 1)) x)).re =
      (inner ℂ (data.perturbation (n + 1) (data.T x))
        ((ContinuousLinearMap.adjoint data.T ^ n) x)).re := by
    rw [pow_succ', mul_apply_eq_comp, data.T.adjoint_inner_right, hcomm]
  rw [hsucc]
  have htop :
      (ContinuousLinearMap.adjoint data.T ^ (n + 1)) (data.T x) =
        ((κ ^ 2 : ℝ) : ℂ) •
          ((ContinuousLinearMap.adjoint data.T ^ n) x) := by
    rw [pow_succ, mul_apply_eq_comp, hsingular, map_smul]
  rw [data.perturbation_eq n, data.perturbation_eq (n + 1)]
  simp only [sub_apply, htop, adjacentPowerDefect,
    ContinuousLinearMap.comp_apply, smul_apply, inner_sub_left,
    inner_smul_left, inner_self_eq_norm_sq_to_K]
  norm_num [pow_two, Complex.mul_re, Complex.ofReal_re, Complex.ofReal_im]
  ring

/-- The exact operator recurrence gives the source's uniform pointwise lower
bound after completing the square and bounding the adjacent-power defect. -/
theorem recurrence_difference_lower_bound
    (data : DilationData (E := E) (K := K))
    {κ : ℝ} (hκ : 1 < κ) (n : ℕ) (x : E)
    (hsingular :
      ContinuousLinearMap.adjoint data.T (data.T x) =
        ((κ ^ 2 : ℝ) : ℂ) • x) :
    -‖data.displacement κ x‖ ^ 2 / (κ ^ 2 - κ) ≤
      κ * recurrenceScalar data x n - recurrenceScalar data x (n + 1) := by
  exact data.recurrence_lower_bound_of_pre_square_identity hκ
    (recurrenceScalar data x) n x
    ((ContinuousLinearMap.adjoint data.T ^ n) x)
    (data.recurrence_difference_identity n x hsingular)

/-- Source Equation (3): iteration of the exact operator recurrence bounds the
first perturbation moment by the common displacement. -/
theorem equation_three_lower_bound
    (data : DilationData (E := E) (K := K))
    {κ : ℝ} (hκ : 1 < κ) {x : E}
    (hx : ‖x‖ = 1)
    (hsingular :
      ContinuousLinearMap.adjoint data.T (data.T x) =
        ((κ ^ 2 : ℝ) : ℂ) • x) :
    -‖data.displacement κ x‖ ^ 2 / (κ * (κ - 1) ^ 2) ≤
      (inner ℂ x (data.perturbation 1 (data.T x))).re := by
  let b : ℝ := ‖data.displacement κ x‖ ^ 2
  let r : ℕ → ℝ := fun _ => -b / (κ ^ 2 - κ)
  have hm : ∀ n, |recurrenceScalar data x n| ≤
      data.bound * (2 + data.bound) :=
    fun n => recurrenceScalar_abs_le data hx n
  have hrec : ∀ n, 1 ≤ n →
      r n ≤ κ * recurrenceScalar data x n -
        recurrenceScalar data x (n + 1) := by
    intro n _hn
    exact data.recurrence_difference_lower_bound hκ n x hsingular
  have herror : ∀ n, 1 ≤ n → -b / (κ ^ 2 - κ) ≤ r n := by
    intro n _hn
    exact le_rfl
  have hbound := recurrence_lower_bound
    (κ := κ) (b := b) (M := data.bound * (2 + data.bound))
    (m := recurrenceScalar data x) (r := r) hκ hm hrec herror
  simpa [b, recurrenceScalar_one] using hbound

end DilationData
end LoristSchwenninger
end CrouzeixConjecture
