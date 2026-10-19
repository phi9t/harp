import Crouzeix.Harp.FiniteHorizonDilation
import Crouzeix.Harp.FiniteHorizonRecurrence
import Crouzeix.LoristSchwenninger.CompletedSquare

/-!
Horizon-local operator recurrence for the Harp finite-dilation route.
-/

noncomputable section

open scoped InnerProduct

namespace CrouzeixConjecture
namespace Harp
namespace FiniteHorizonDilationData

variable {E K : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace ℂ E]
  [CompleteSpace E] [Nontrivial E]
variable [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]
variable {core : CommutingPerturbationData E} {N : ℕ}

/-- The common displacement isolated from every adjacent-power defect. -/
def displacement (data : FiniteHorizonDilationData core K N)
    (κ : ℝ) (x : E) : K :=
  ContinuousLinearMap.adjoint data.Q (data.V (core.T x)) -
    (κ : ℂ) • data.V x

/-- The nonnegative displacement scalar. -/
def displacementSq (data : FiniteHorizonDilationData core K N)
    (κ : ℝ) (x : E) : ℝ :=
  ‖data.displacement κ x‖ ^ 2

omit [Nontrivial E] in
theorem displacementSq_nonneg
    (data : FiniteHorizonDilationData core K N) (κ : ℝ) (x : E) :
    0 ≤ data.displacementSq κ x := by
  exact sq_nonneg _

omit [Nontrivial E] in
/-- The finite-horizon dilation map preserves vector norms. -/
theorem V_apply_norm (data : FiniteHorizonDilationData core K N) (x : E) :
    ‖data.V x‖ = ‖x‖ := by
  exact data.isometry.norm_map_of_map_zero data.V.map_zero x

/-- The finite-horizon dilation map has operator norm one. -/
theorem V_norm (data : FiniteHorizonDilationData core K N) :
    ‖data.V‖ = 1 := by
  let Vi : E →ₗᵢ[ℂ] K :=
    { data.V.toLinearMap with
      norm_map' := data.isometry.norm_map_of_map_zero data.V.map_zero }
  have hmap : Vi.toContinuousLinearMap = data.V := by
    ext x
    rfl
  rw [← hmap]
  exact Vi.norm_toContinuousLinearMap

/-- The adjoint of the dilation map is contractive. -/
theorem V_adjoint_apply_norm_le
    (data : FiniteHorizonDilationData core K N) (y : K) :
    ‖ContinuousLinearMap.adjoint data.V y‖ ≤ ‖y‖ := by
  calc
    ‖ContinuousLinearMap.adjoint data.V y‖
        ≤ ‖ContinuousLinearMap.adjoint data.V‖ * ‖y‖ :=
      (ContinuousLinearMap.adjoint data.V).le_opNorm y
    _ = ‖y‖ := by
      rw [(ContinuousLinearMap.adjoint (𝕜 := ℂ)).norm_map, data.V_norm, one_mul]

omit [Nontrivial E] in
/-- The adjoint of the horizon-local contraction is contractive. -/
theorem Q_adjoint_apply_norm_le
    (data : FiniteHorizonDilationData core K N) (y : K) :
    ‖ContinuousLinearMap.adjoint data.Q y‖ ≤ ‖y‖ := by
  calc
    ‖ContinuousLinearMap.adjoint data.Q y‖
        ≤ ‖ContinuousLinearMap.adjoint data.Q‖ * ‖y‖ :=
      (ContinuousLinearMap.adjoint data.Q).le_opNorm y
    _ ≤ 1 * ‖y‖ := by
      rw [(ContinuousLinearMap.adjoint (𝕜 := ℂ)).norm_map]
      exact mul_le_mul_of_nonneg_right data.contraction (norm_nonneg y)
    _ = ‖y‖ := one_mul _

omit [Nontrivial E] in
/-- Every adjoint power of the horizon-local contraction is contractive. -/
theorem Q_adjoint_power_apply_norm_le
    (data : FiniteHorizonDilationData core K N) (n : ℕ) (y : K) :
    ‖((ContinuousLinearMap.adjoint data.Q) ^ n) y‖ ≤ ‖y‖ := by
  induction n with
  | zero => simp
  | succ n ih =>
      rw [pow_succ', mul_apply_eq_comp]
      exact (data.Q_adjoint_apply_norm_le _).trans ih

/-- The adjacent defect between two compressed powers.  Its factorization is
horizon-independent; only later substitution of perturbation identities is
guarded by the horizon. -/
def adjacentPowerDefect (data : FiniteHorizonDilationData core K N)
    (κ : ℝ) (n : ℕ) (x : E) : E :=
  ((LoristSchwenninger.doubledCompressedAdjointPower data.V data.Q (n + 1)).comp
      core.T -
    (κ : ℂ) •
      LoristSchwenninger.doubledCompressedAdjointPower data.V data.Q n) x

omit [Nontrivial E] in
/-- Factoring adjacent powers isolates the common displacement. -/
theorem adjacentPowerDefect_factorization
    (data : FiniteHorizonDilationData core K N)
    (κ : ℝ) (n : ℕ) (x : E) :
    data.adjacentPowerDefect κ n x =
      (2 : ℂ) • ContinuousLinearMap.adjoint data.V
        (((ContinuousLinearMap.adjoint data.Q) ^ n)
          (data.displacement κ x)) := by
  simp only [adjacentPowerDefect,
    LoristSchwenninger.doubledCompressedAdjointPower,
    LoristSchwenninger.compressedAdjointPower, displacement, sub_apply,
    ContinuousLinearMap.comp_apply, smul_apply]
  rw [pow_succ, mul_apply_eq_comp]
  simp only [map_sub, map_smul]
  module

/-- The adjacent-power defect is uniformly controlled by the common
displacement. -/
theorem adjacentPowerDefect_norm_le
    (data : FiniteHorizonDilationData core K N)
    (κ : ℝ) (n : ℕ) (x : E) :
    ‖data.adjacentPowerDefect κ n x‖ ≤
      2 * ‖data.displacement κ x‖ := by
  rw [data.adjacentPowerDefect_factorization κ n x, norm_smul]
  have htwo : ‖(2 : ℂ)‖ = (2 : ℝ) := by norm_num
  rw [htwo]
  refine mul_le_mul_of_nonneg_left ?_ (by norm_num)
  exact (data.V_adjoint_apply_norm_le _).trans
    (data.Q_adjoint_power_apply_norm_le n _)

omit [Nontrivial E] in
private theorem recurrenceScalar_eq_inner
    (core : CommutingPerturbationData E) (n : ℕ) (x : E) :
    core.recurrenceScalar x n =
      (inner ℂ (core.perturbation n x)
        ((ContinuousLinearMap.adjoint core.T ^ n) x)).re := by
  rw [CommutingPerturbationData.recurrenceScalar,
    (core.commutes_with_target n).pow_right n |>.eq]
  simp only [mul_apply_eq_comp]
  rw [← (core.T ^ n).adjoint_inner_left (core.perturbation n x) x]
  rw [← ContinuousLinearMap.star_eq_adjoint (core.T ^ n), star_pow,
    ContinuousLinearMap.star_eq_adjoint core.T]
  exact @inner_re_symm ℂ E _ _ _
    ((ContinuousLinearMap.adjoint core.T ^ n) x)
    (core.perturbation n x)

omit [Nontrivial E] in
/-- The denominator-free recurrence identity at an adjacent pair realized by
the finite dilation witness. -/
theorem recurrence_difference_identity
    (data : FiniteHorizonDilationData core K N)
    {κ : ℝ} (n : ℕ) (hn : n + 1 ≤ N + 1) (x : E)
    (hsingular :
      ContinuousLinearMap.adjoint core.T (core.T x) =
        ((κ ^ 2 : ℝ) : ℂ) • x) :
    κ * core.recurrenceScalar x n - core.recurrenceScalar x (n + 1) =
      (κ ^ 2 - κ) * ‖(ContinuousLinearMap.adjoint core.T ^ n) x‖ ^ 2 -
        (inner ℂ (data.adjacentPowerDefect κ n x)
          ((ContinuousLinearMap.adjoint core.T ^ n) x)).re := by
  rw [recurrenceScalar_eq_inner core n x,
    recurrenceScalar_eq_inner core (n + 1) x]
  have hcomm :
      core.T (core.perturbation (n + 1) x) =
        core.perturbation (n + 1) (core.T x) := by
    have h := congrArg (fun A : E →L[ℂ] E => A x)
      (core.commutes_with_target (n + 1)).eq
    simpa [mul_apply_eq_comp] using h.symm
  have hsucc :
      (inner ℂ (core.perturbation (n + 1) x)
        ((ContinuousLinearMap.adjoint core.T ^ (n + 1)) x)).re =
      (inner ℂ (core.perturbation (n + 1) (core.T x))
        ((ContinuousLinearMap.adjoint core.T ^ n) x)).re := by
    rw [pow_succ', mul_apply_eq_comp, core.T.adjoint_inner_right, hcomm]
  rw [hsucc]
  have htop :
      (ContinuousLinearMap.adjoint core.T ^ (n + 1)) (core.T x) =
        ((κ ^ 2 : ℝ) : ℂ) •
          ((ContinuousLinearMap.adjoint core.T ^ n) x) := by
    rw [pow_succ, mul_apply_eq_comp, hsingular, map_smul]
  have hn' : n ≤ N + 1 := le_trans (Nat.le_succ n) hn
  rw [data.perturbation_eq n hn', data.perturbation_eq (n + 1) hn]
  simp only [sub_apply, htop, adjacentPowerDefect,
    ContinuousLinearMap.comp_apply, smul_apply, inner_sub_left,
    inner_smul_left, inner_self_eq_norm_sq_to_K]
  norm_num [pow_two, Complex.mul_re, Complex.ofReal_re, Complex.ofReal_im]
  ring

/-- Completing the square yields the common displacement lower bound on each
recurrence step inside the realized horizon. -/
theorem recurrence_difference_lower_bound
    (data : FiniteHorizonDilationData core K N)
    {κ : ℝ} (hκ : 1 < κ) (n : ℕ) (_hnpos : 1 ≤ n) (hn : n ≤ N)
    (x : E)
    (hsingular :
      ContinuousLinearMap.adjoint core.T (core.T x) =
        ((κ ^ 2 : ℝ) : ℂ) • x) :
    -data.displacementSq κ x / (κ ^ 2 - κ) ≤
      κ * core.recurrenceScalar x n - core.recurrenceScalar x (n + 1) := by
  have ha : 0 < κ ^ 2 - κ := by nlinarith
  have hpre := data.recurrence_difference_identity n (by omega) x hsingular
  have hcompleted := LoristSchwenninger.pre_square_lower_bound ha
    ((ContinuousLinearMap.adjoint core.T ^ n) x)
    (data.adjacentPowerDefect κ n x) hpre
  have hdefect := data.adjacentPowerDefect_norm_le κ n x
  have hsq :
      ‖data.adjacentPowerDefect κ n x‖ ^ 2 ≤
        (2 * ‖data.displacement κ x‖) ^ 2 :=
    pow_le_pow_left₀ (norm_nonneg _) hdefect 2
  have herror :
      -data.displacementSq κ x / (κ ^ 2 - κ) ≤
        -‖data.adjacentPowerDefect κ n x‖ ^ 2 /
          (4 * (κ ^ 2 - κ)) := by
    rw [div_le_div_iff₀ ha (mul_pos (by norm_num) ha)]
    simp only [displacementSq]
    nlinarith
  exact herror.trans hcompleted

/-- Finite source Equation (3): iterating all recurrence steps realized by a
horizon-`N` witness retains the terminal moment explicitly. -/
theorem equation_three_finite_lower_bound
    (data : FiniteHorizonDilationData core K N)
    {κ : ℝ} (hκ : 1 < κ) (x : E)
    (hsingular :
      ContinuousLinearMap.adjoint core.T (core.T x) =
        ((κ ^ 2 : ℝ) : ℂ) • x) :
    (κ⁻¹) ^ N * core.recurrenceScalar x (N + 1) +
        (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) *
          (-data.displacementSq κ x / (κ ^ 2 - κ)) ≤
      core.recurrenceScalar x 1 := by
  let c : ℝ := -data.displacementSq κ x / (κ ^ 2 - κ)
  let m : ℕ → ℝ := core.recurrenceScalar x
  let mExt : ℕ → ℝ := fun k =>
    if k ≤ N + 1 then m k
    else κ ^ (k - (N + 1)) * m (N + 1)
  let r : ℕ → ℝ := fun k => if k ≤ N then c else 0
  have hc : c ≤ 0 := by
    dsimp [c]
    exact div_nonpos_of_nonpos_of_nonneg
      (neg_nonpos.mpr (data.displacementSq_nonneg κ x))
      (by nlinarith : 0 ≤ κ ^ 2 - κ)
  have hrec : ∀ n, 1 ≤ n →
      r n ≤ κ * mExt n - mExt (n + 1) := by
    intro n hnpos
    by_cases hn : n ≤ N
    · have hn1 : n + 1 ≤ N + 1 := by omega
      have hn0 : n ≤ N + 1 := by omega
      simpa [r, mExt, m, c, hn, hn0, hn1] using
        data.recurrence_difference_lower_bound hκ n hnpos hn x hsingular
    · have hr : r n = 0 := by simp [r, hn]
      rw [hr]
      by_cases hn1 : n ≤ N + 1
      · have heq : n = N + 1 := by omega
        subst n
        simp [mExt]
      · have hnnext : ¬n + 1 ≤ N + 1 := by omega
        have hexp : n + 1 - (N + 1) = (n - (N + 1)) + 1 := by omega
        simp only [mExt, if_neg hn1, if_neg hnnext, hexp, pow_succ]
        ring_nf
        norm_num
  have herror : ∀ n, 1 ≤ n → c ≤ r n := by
    intro n _hnpos
    by_cases hn : n ≤ N
    · simp [r, hn]
    · simpa [r, hn] using hc
  have hiter := LoristSchwenninger.recurrence_finite_iteration_of_constant_error
    (κ := κ) (c := c) (m := mExt) (r := r)
    (lt_trans zero_lt_one hκ) hrec herror N
  simpa [mExt, m, c] using hiter

omit [Nontrivial E] in
/-- Finite source Equation (4): every horizon witness gives the same upper
bound on its squared displacement at a unit top singular vector. -/
theorem displacementSq_le
    (data : FiniteHorizonDilationData core K N)
    {κ : ℝ} {x : E}
    (hx : ‖x‖ = 1)
    (hκ : κ = ‖core.T‖)
    (hsingular :
      ContinuousLinearMap.adjoint core.T (core.T x) =
        ((κ ^ 2 : ℝ) : ℂ) • x) :
    data.displacementSq κ x ≤
      2 * κ ^ 2 - κ * core.recurrenceScalar x 1 - κ ^ 3 := by
  let A : K := ContinuousLinearMap.adjoint data.Q (data.V (core.T x))
  have hA_norm : ‖A‖ ≤ κ := by
    calc
      ‖A‖ ≤ ‖data.V (core.T x)‖ := data.Q_adjoint_apply_norm_le _
      _ = ‖core.T x‖ := data.V_apply_norm _
      _ ≤ ‖core.T‖ * ‖x‖ := core.T.le_opNorm x
      _ = κ := by rw [hx, mul_one, hκ]
  have hA_sq : ‖A‖ ^ 2 ≤ κ ^ 2 := by
    nlinarith [norm_nonneg A]
  have hE1 :
      core.perturbation 1 (core.T x) =
        (2 : ℂ) • ContinuousLinearMap.adjoint data.V A -
          ContinuousLinearMap.adjoint core.T (core.T x) := by
    have h := congrArg (fun S : E →L[ℂ] E => S (core.T x))
      (data.perturbation_eq 1 (by omega))
    simpa [LoristSchwenninger.doubledCompressedAdjointPower,
      LoristSchwenninger.compressedAdjointPower, A] using h
  have hm0 :
      core.recurrenceScalar x 1 =
        (inner ℂ x (core.perturbation 1 (core.T x))).re := by
    simp [CommutingPerturbationData.recurrenceScalar, mul_apply_eq_comp]
  have hm :
      core.recurrenceScalar x 1 =
        2 * (inner ℂ A (data.V x)).re - κ ^ 2 := by
    have hreal :
        (inner ℂ x (core.perturbation 1 (core.T x))).re =
          (inner ℂ (core.perturbation 1 (core.T x)) x).re :=
      @inner_re_symm ℂ E _ _ _ x (core.perturbation 1 (core.T x))
    rw [hm0, hreal, hE1, inner_sub_left, inner_smul_left,
      ContinuousLinearMap.adjoint_inner_left, hsingular, inner_smul_left,
      inner_self_eq_norm_sq_to_K, hx]
    norm_num [Complex.mul_re, pow_two]
  have hb :
      data.displacementSq κ x =
        ‖A‖ ^ 2 - 2 * κ * (inner ℂ A (data.V x)).re + κ ^ 2 := by
    rw [displacementSq, displacement, show
      ContinuousLinearMap.adjoint data.Q (data.V (core.T x)) = A by rfl,
      @norm_sub_sq ℂ, norm_smul, data.V_apply_norm, hx, mul_one,
      inner_smul_right]
    simp only [Complex.norm_real, Real.norm_eq_abs]
    rw [sq_abs]
    norm_num [Complex.mul_re]
    ring
  calc
    data.displacementSq κ x =
        ‖A‖ ^ 2 - 2 * κ * (inner ℂ A (data.V x)).re + κ ^ 2 := hb
    _ ≤ κ ^ 2 - 2 * κ * (inner ℂ A (data.V x)).re + κ ^ 2 := by
      linarith
    _ = 2 * κ ^ 2 - κ * core.recurrenceScalar x 1 - κ ^ 3 := by
      rw [hm]
      ring

end FiniteHorizonDilationData
end Harp
end CrouzeixConjecture
