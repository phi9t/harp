import Crouzeix.Harp.Consequences

/-! Teaching interfaces for the Harp finite-horizon route. Five interfaces
below are explicit reexports; finite dilation existence packages a provider
witness into `Nonempty`, rather than reproving its construction.
The exercises isolate coordinate projection, normalization, finite induction,
sign control, rescaling, and the scalar limiting argument. -/

noncomputable section

namespace CrouzeixTextbook.Part06
open CrouzeixConjecture
open MeasureTheory Set
open scoped InnerProductSpace Matrix Matrix.Norms.L2Operator ComplexOrder MatrixOrder

section Cubature
variable {i : Type*} {n : Type} [TopologicalSpace i] [CompactSpace i]
  [MeasurableSpace i] [OpensMeasurableSpace i]
  [Fintype n] [DecidableEq n] {Ω : Set ℂ}

universe u

set_option linter.defProp false in
/-- Reexport: a single positive cubature preserves powers through `N + 1`. -/
def harp_positive_moment_cubature := @Harp.exists_positive_matrix_moment_cubature.{u}

/-- Existence packaging: the Cauchy formula and normalized boundary polynomial produce
a finite witness; its type retains the horizon-independent core. -/
theorem harp_finite_dilation_exists [Nonempty n] (N : ℕ)
    (Gamma : ParametricConvexBoundary (i := i) Ω)
    (μ : Measure i) [IsFiniteMeasure μ] [NeZero μ]
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Ω) (q : Polynomial ℂ)
    (hq : ∀ z ∈ closure Ω, ‖Polynomial.eval z q‖ ≤ 1)
    (hCauchy : HasParametricPolynomialCauchyFormula Gamma μ B) :
    Nonempty (Harp.FiniteAtomicL2DilationWitness
      (Harp.finiteHorizonPolynomialCore Gamma μ B hWB q hq) N) :=
  ⟨Harp.finiteAtomicL2DilationWitness N Gamma μ B hWB q hq hCauchy⟩
end Cubature

section Dilation
variable {E K : Type*} [NormedAddCommGroup E] [InnerProductSpace ℂ E]
  [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E]
  [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]

omit [FiniteDimensional ℂ E] in
/-- Reexport: the finite recurrence retains the terminal moment. -/
theorem harp_finite_recurrence {core : Harp.CommutingPerturbationData E} {N : ℕ}
    (data : Harp.FiniteHorizonDilationData core K N) {κ : ℝ} (hκ : 1 < κ)
    (x : E) (hsingular : ContinuousLinearMap.adjoint core.T (core.T x) =
      ((κ ^ 2 : ℝ) : ℂ) • x) :
    (κ⁻¹) ^ N * core.recurrenceScalar x (N + 1) +
      (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) *
        (-data.displacementSq κ x / (κ ^ 2 - κ)) ≤ core.recurrenceScalar x 1 :=
  data.equation_three_finite_lower_bound hκ x hsingular

set_option linter.defProp false in
/-- Reexport: every horizon has its own dilation space and witness. -/
def harp_normalized_norm_two :=
  @Harp.norm_target_le_two_of_finiteHorizonDilationData.{u_1, u_2}
end Dilation

/-- Reexport of the Harp polynomial theorem in its exact matrix universe. -/
theorem harp_polynomial_constant_two {n : Type}
    [Fintype n] [DecidableEq n] [Nonempty n] : MainTheoremStatement (n := n) :=
  Harp.harpFiniteHorizonMainTheorem

/-- Reexport of the Harp closed numerical-range spectral-set consequence. -/
theorem harp_two_spectral_set {H : Type*}
    [NormedAddCommGroup H] [InnerProductSpace ℂ H] [CompleteSpace H] [Nontrivial H]
    (A : H →L[ℂ] H) : ClosedOperatorNumericalRangeIsTwoSpectralSet A :=
  harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet A

namespace Exercises.Chapter36

/-- E01. Project one moment from a preserved tuple, before commuting an
integral with evaluation. -/
theorem exercise_01_solution {ι : Type*} [Fintype ι] {N : ℕ}
    {n : Type} [Fintype n] [DecidableEq n]
    (weights : ι → ℝ) (moments : ι → Fin (N + 2) → SquareMatrix n)
    (target : Fin (N + 2) → SquareMatrix n)
    (hpreserved : ∑ j, weights j • moments j = target) (k : Fin (N + 2)) :
    ∑ j, weights j • moments j k = target k := by
  have hcoordinate := congrArg (fun f : Fin (N + 2) → SquareMatrix n => f k) hpreserved
  simpa using hcoordinate

/-- E02. Positive atomic matrix densities sum to `2 I`; their square-root
embedding therefore preserves norms. The inner-product premise exposes the
normalized density calculation, not its norm-preservation conclusion. -/
theorem exercise_02_solution {n : Type} [Fintype n] [DecidableEq n] {K : Type*}
    [NormedAddCommGroup K] [InnerProductSpace ℂ K]
    {ι : Type*} [Fintype ι] (D : ι → {A : SquareMatrix n // A.PosSemidef})
    (hmass : ∑ j, (D j).val = (2 : ℂ) • (1 : SquareMatrix n))
    (V : EuclideanVector n →ₗ[ℂ] K)
    (hinner : ∀ x, inner ℂ (V x) (V x) =
      (2 : ℂ)⁻¹ * inner ℂ x (euclideanOperator (∑ j, (D j).val) x))
    (x : EuclideanVector n) :
    ‖V x‖ = ‖x‖ := by
  have hi : inner ℂ (V x) (V x) = inner ℂ x x := by
    simpa [hmass, inner_smul_right] using hinner x
  have hs : ‖V x‖ ^ 2 = ‖x‖ ^ 2 := by
    rw [norm_sq_eq_re_inner (𝕜 := ℂ), norm_sq_eq_re_inner (𝕜 := ℂ), hi]
  nlinarith [norm_nonneg (V x), norm_nonneg x]

/-- E03. Weighted finite telescoping, with no discarded terminal term. -/
theorem exercise_03_solution (r c : ℝ) (hr : 0 ≤ r) (m : ℕ → ℝ) (N : ℕ)
    (hstep : ∀ j, j < N → r * m (j + 2) + r * c ≤ m (j + 1)) :
    r ^ N * m (N + 1) + (∑ j ∈ Finset.range N, r ^ (j + 1)) * c ≤ m 1 := by
  induction N with
  | zero => simp
  | succ N ih =>
    have hprev := ih (fun j hj => hstep j (by omega))
    have hlast := mul_le_mul_of_nonneg_left (hstep N (by omega)) (pow_nonneg hr N)
    rw [Finset.sum_range_succ, pow_succ]
    nlinarith

/-- E04. An upper bound on the displacement becomes a lower bound on its
negative error; the denominator must be positive. -/
theorem exercise_04_solution {b C d w t m : ℝ}
    (hb : b ≤ C) (hd : 0 < d) (hw : 0 ≤ w)
    (hfinite : t + w * (-b / d) ≤ m) : t + w * (-C / d) ≤ m := by
  have hcoeff : -C / d ≤ -b / d :=
    div_le_div_of_nonneg_right (neg_le_neg hb) hd.le
  exact (add_le_add le_rfl (mul_le_mul_of_nonneg_left hcoeff hw)).trans hfinite

/-- E05. Undo a strictly positive normalization at the scalar norm level. -/
theorem exercise_05_solution {a s : ℝ} (hs : 0 < s) (hnormalized : a / s ≤ 2) :
    a ≤ 2 * s := by
  exact (div_le_iff₀ hs).mp hnormalized

/-- E06. Bounded terminal moments vanish geometrically. Only the scalar limit
and scalar endpoint support lemmas are used, never a terminal operator theorem. -/
theorem exercise_06_solution {κ C M : ℝ} {m : ℕ → ℝ}
    (hκ : 1 < κ) (hC : 0 ≤ C) (hbounded : ∀ j, |m j| ≤ M)
    (hfinite : ∀ N, (κ⁻¹) ^ N * m (N + 1) +
      (∑ j ∈ Finset.range N, (κ⁻¹) ^ (j + 1)) * (-C / (κ ^ 2 - κ)) ≤ m 1)
    (hupper : C ≤ 2 * κ ^ 2 - κ * m 1 - κ ^ 3) : κ ≤ 2 := by
  have hterminal := LoristSchwenninger.bounded_recurrence_terminal_tendsto_zero
    hκ hbounded
  have hweights := LoristSchwenninger.inverse_power_weight_sum_tendsto hκ
  have hleft := hterminal.add (hweights.mul_const (-C / (κ ^ 2 - κ)))
  have hlimit : (-C / (κ ^ 2 - κ)) / (κ - 1) ≤ m 1 := by
    have hle := le_of_tendsto' hleft hfinite
    simpa [div_eq_mul_inv, mul_comm] using hle
  have hk0 : κ ≠ 0 := ne_of_gt (lt_trans zero_lt_one hκ)
  have hk1 : κ - 1 ≠ 0 := sub_ne_zero.mpr hκ.ne'
  have hden : κ ^ 2 - κ ≠ 0 := by
    rw [show κ ^ 2 - κ = κ * (κ - 1) by ring]
    exact mul_ne_zero hk0 hk1
  have halgebra : (-C / (κ ^ 2 - κ)) / (κ - 1) =
      -C / (κ * (κ - 1) ^ 2) := by field_simp [hk0, hk1, hden]
  rw [halgebra] at hlimit
  exact LoristSchwenninger.scalar_endpoint_le_two hC (lt_trans zero_lt_one hκ)
    (sq_pos_of_pos (sub_pos.mpr hκ)) hlimit hupper

end Exercises.Chapter36
end CrouzeixTextbook.Part06
