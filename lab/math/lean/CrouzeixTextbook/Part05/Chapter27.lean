import CrouzeixConjecture.DoubleLayerPositiveMap
import Mathlib.NumberTheory.Real.Irrational

namespace CrouzeixTextbook.Part05
open CrouzeixConjecture MeasureTheory
open scoped BoundedContinuousFunction InnerProductSpace Matrix
  Matrix.Norms.L2Operator

noncomputable section

/-- The principal real square root of two is on the nonnegative branch. -/
theorem sqrt_two_nonnegative : 0 ≤ Real.sqrt 2 := by
  exact Real.sqrt_nonneg _

/-- Squaring the principal real square root recovers two. -/
theorem sqrt_two_squared : (Real.sqrt 2) ^ 2 = 2 := by
  exact Real.sq_sqrt (by norm_num)

/-- The upper root of `x² - 2x - 1` is positive. -/
theorem one_plus_sqrt_two_positive : 0 < 1 + Real.sqrt 2 := by
  have hsqrt := Real.sqrt_nonneg 2
  linarith

/-- The square root appearing in the barrier is genuinely irrational; this
supports the historical and arithmetic discussion but is not used as a bound. -/
theorem sqrt_two_irrational : Irrational (Real.sqrt 2) := by
  exact irrational_sqrt_two

/-- The quadratic inequality itself forces the upper-root bound. No separate
nonnegativity hypothesis on `x` is needed. -/
theorem one_plus_sqrt_two_barrier {x : ℝ}
    (hquad : x ^ 2 ≤ 2 * x + 1) : x ≤ 1 + Real.sqrt 2 := by
  have hsqrt0 : 0 ≤ Real.sqrt 2 := Real.sqrt_nonneg _
  have hsqrt2 : (Real.sqrt 2) ^ 2 = 2 :=
    Real.sq_sqrt (by norm_num)
  have hfactored :
      (x - (1 + Real.sqrt 2)) * (x - (1 - Real.sqrt 2)) ≤ 0 := by
    nlinarith
  by_contra hbound
  have hleft : 0 < x - (1 + Real.sqrt 2) := by linarith
  have hright : 0 < x - (1 - Real.sqrt 2) := by linarith
  exact (not_lt_of_ge hfactored) (mul_pos hleft hright)

/-- Independent norm bounds on two summands give the scalar expression that
appears on the right side of the classical quadratic estimate. -/
theorem triangle_barrier_kernel
    {E : Type*} [SeminormedAddCommGroup E]
    (X Y : E) (kappa : ℝ)
    (hX : ‖X‖ ≤ 2 * kappa) (hY : ‖Y‖ ≤ 1) :
    ‖X + Y‖ ≤ 2 * kappa + 1 := by
  calc
    ‖X + Y‖ ≤ ‖X‖ + ‖Y‖ := norm_add_le X Y
    _ ≤ 2 * kappa + 1 := add_le_add hX hY

/-- If the positive-map term and its companion decompose `Tᴴ * T`, their
separate norm estimates imply the classical quadratic obstruction. -/
theorem positive_map_norm_kernel
    {i n : Type*} [TopologicalSpace i] [MeasurableSpace i]
    [OpensMeasurableSpace i] [Fintype n] [DecidableEq n]
    {mu : Measure i}
    (D : PositiveBoundaryDensity (n := n) mu) (h : i →ᵇ ℂ)
    (T R : SquareMatrix n)
    (hdecomp : Tᴴ * T = boundaryPhiCLM D h + R)
    (hpositiveMap :
      ((2 : ℝ)⁻¹ * ∫ x, ‖D.density x‖ ∂mu) * ‖h‖ ≤ 2 * ‖T‖)
    (hcompanion : ‖R‖ ≤ 1) :
    ‖T‖ ^ 2 ≤ 2 * ‖T‖ + 1 := by
  have hPhi : ‖boundaryPhiCLM D h‖ ≤ 2 * ‖T‖ :=
    (boundaryPhi_norm_le D h).trans hpositiveMap
  calc
    ‖T‖ ^ 2 = ‖Tᴴ * T‖ := by
      rw [Matrix.l2_opNorm_conjTranspose_mul_self]
      ring
    _ = ‖boundaryPhiCLM D h + R‖ := congrArg norm hdecomp
    _ ≤ 2 * ‖T‖ + 1 :=
      triangle_barrier_kernel (boundaryPhiCLM D h) R ‖T‖ hPhi hcompanion

namespace Exercises.Chapter27

theorem exercise_01_solution :
    ∀ κ : ℝ, 0 ≤ κ → κ ^ 2 ≤ 2 * κ + 1 →
      κ ≤ 1 + Real.sqrt 2 := by
  intro κ _hκ hquad
  have hsqrt0 : 0 ≤ Real.sqrt 2 := Real.sqrt_nonneg _
  have hsqrt2 : (Real.sqrt 2) ^ 2 = 2 :=
    Real.sq_sqrt (by norm_num)
  have hfactored :
      (κ - (1 + Real.sqrt 2)) *
          (κ - (1 - Real.sqrt 2)) ≤ 0 := by
    nlinarith
  by_contra hbound
  have hleft : 0 < κ - (1 + Real.sqrt 2) := by linarith
  have hright : 0 < κ - (1 - Real.sqrt 2) := by linarith
  exact (not_lt_of_ge hfactored) (mul_pos hleft hright)

theorem exercise_02_solution :
    2 < (1 : ℝ) + Real.sqrt 2 ∧
      (1 : ℝ) + Real.sqrt 2 < 5 / 2 := by
  have hsqrt0 : 0 ≤ Real.sqrt 2 := Real.sqrt_nonneg _
  have hsqrt2 : (Real.sqrt 2) ^ 2 = 2 :=
    Real.sq_sqrt (by norm_num)
  constructor <;> nlinarith

theorem exercise_03_solution :
    ∀ κ : ℝ, κ ^ 2 ≤ 2 * κ + 1 →
      (κ - 1) ^ 2 ≤ 2 ∧ κ ≤ 1 + Real.sqrt 2 := by
  intro κ hquad
  have hsqrt0 : 0 ≤ Real.sqrt 2 := Real.sqrt_nonneg _
  have hsqrt2 : (Real.sqrt 2) ^ 2 = 2 :=
    Real.sq_sqrt (by norm_num)
  have hsquare : (κ - 1) ^ 2 ≤ 2 := by nlinarith
  refine ⟨hsquare, ?_⟩
  by_contra hbound
  have hstrict : Real.sqrt 2 < κ - 1 := by linarith
  have hsum : 0 < (κ - 1) + Real.sqrt 2 := by linarith
  have hproduct :
      0 < ((κ - 1) - Real.sqrt 2) *
        ((κ - 1) + Real.sqrt 2) :=
    mul_pos (by linarith) hsum
  nlinarith

theorem exercise_04_solution :
    ∃ a b : ℂ, ‖a‖ = 1 ∧ ‖b‖ = Real.sqrt 2 ∧
      ‖a + b‖ = 1 + Real.sqrt 2 := by
  refine ⟨(1 : ℂ), (Real.sqrt 2 : ℂ), by norm_num, ?_, ?_⟩
  · rw [Complex.norm_real, Real.norm_of_nonneg (Real.sqrt_nonneg _)]
  · rw [show (1 : ℂ) + (Real.sqrt 2 : ℂ) =
        ((1 + Real.sqrt 2 : ℝ) : ℂ) by norm_num]
    rw [Complex.norm_real, Real.norm_of_nonneg]
    positivity

theorem exercise_05_solution :
    ∀ (E : Type) [NormedAddCommGroup E] [InnerProductSpace ℂ E]
      (x y : E), ⟪x, y⟫_ℂ = 0 →
        ‖x + y‖ ^ 2 = ‖x‖ ^ 2 + ‖y‖ ^ 2 := by
  intro E _ _ x y hxy
  simpa only [pow_two] using
    norm_add_sq_eq_norm_sq_add_norm_sq_of_inner_eq_zero x y hxy

theorem exercise_06_solution :
    ∀ κ : ℝ, κ ^ 2 ≤ 2 * κ + 1 →
      (κ - (1 + Real.sqrt 2)) *
          (κ - (1 - Real.sqrt 2)) ≤ 0 ∧
        κ ≤ 1 + Real.sqrt 2 ∧
        ∀ (E : Type) [NormedAddCommGroup E] [InnerProductSpace ℂ E]
          (T : E →L[ℂ] E), κ = ‖T‖ → 0 ≤ κ := by
  intro κ hquad
  have hsqrt0 : 0 ≤ Real.sqrt 2 := Real.sqrt_nonneg _
  have hsqrt2 : (Real.sqrt 2) ^ 2 = 2 :=
    Real.sq_sqrt (by norm_num)
  have hfactored :
      (κ - (1 + Real.sqrt 2)) *
          (κ - (1 - Real.sqrt 2)) ≤ 0 := by
    nlinarith
  have hbarrier : κ ≤ 1 + Real.sqrt 2 := by
    by_contra hbound
    have hleft : 0 < κ - (1 + Real.sqrt 2) := by linarith
    have hright : 0 < κ - (1 - Real.sqrt 2) := by linarith
    exact (not_lt_of_ge hfactored) (mul_pos hleft hright)
  refine ⟨hfactored, hbarrier, ?_⟩
  intro E _ _ T hκ
  rw [hκ]
  exact norm_nonneg T

end Exercises.Chapter27

/-- The signed correlation term retained before the triangle relaxation. -/
theorem barrier_cross_term_identity
    {E : Type*} [NormedAddCommGroup E] [InnerProductSpace ℂ E]
    (u v : E) :
    ‖u - v‖ ^ 2 =
      ‖u‖ ^ 2 + ‖v‖ ^ 2 - 2 * RCLike.re ⟪u, v⟫_ℂ := by
  rw [norm_sub_sq (𝕜 := ℂ)]
  ring_nf

end
end CrouzeixTextbook.Part05
