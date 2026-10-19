import CrouzeixConjecture.RadialOrientation
import CrouzeixConjecture.ParallelRadialDifferentiability

namespace CrouzeixTextbook.Part02
open CrouzeixConjecture
open scoped InnerProductSpace

attribute [local instance] Complex.finrank_real_complex_fact

/-- CFT-12-001: the real inner product on `ℂ` read in real coordinates. Proved here
rather than re-exported: the maintained provider `complexRealInner_eq_re_mul_conj` is
an exact eta-alias of Mathlib's `Complex.inner`, so the receipt classifies it as
`direct-alias` and it cannot serve as an underlying declaration. -/
theorem real_inner_complex_coordinates (z w : ℂ) :
    ⟪z, w⟫_ℝ = z.re * w.re + z.im * w.im := by
  rw [complexRealInner_eq_re_mul_conj, Complex.mul_re]
  simp only [Complex.star_def, Complex.conj_re, Complex.conj_im, mul_neg, sub_neg_eq_add]
  ring
set_option linter.defProp false in
def area_form_quarter_turn := @complexAreaForm_I_mul_right
set_option linter.defProp false in
def oriented_radial_tangent_positive := @complexAreaForm_radialTangent_pos
set_option linter.defProp false in
def oriented_tangent_formula := @radialTangent_eq_I_mul_normal_mul_norm
set_option linter.defProp false in
def boundary_parameter_is_smooth := @contDiff_one_parallelRadialParameterPoint
set_option linter.defProp false in
def boundary_slice_derivative := @parallelRadialParameterPoint_slice_hasDerivAt

namespace Exercises.Chapter12

/-- CFT-12-E01. -/
theorem exercise_01_solution (z w : ℂ) :
    Complex.orientation.areaForm z (Complex.I * w) = ⟪z, w⟫_ℝ ∧
      ⟪Complex.I * z, w⟫_ℝ = Complex.orientation.areaForm z w :=
  ⟨complexAreaForm_I_mul_right z w, complexRealInner_I_mul_left z w⟩

/-- CFT-12-E02. -/
theorem exercise_02_solution :
    Complex.orientation.areaForm 1 Complex.I = 1 ∧
      ⟪(1 : ℂ), Complex.I⟫_ℝ = 0 := by
  constructor
  · rw [Complex.areaForm]; simp
  · rw [complexRealInner_eq_re_mul_conj]; simp

/-- CFT-12-E03. -/
theorem exercise_03_solution (c : ℂ) (t s₀ s₁ : ℝ) :
    parallelRadialParameterPoint c (t, s₁) - parallelRadialParameterPoint c (t, s₀)
      = (s₁ - s₀) • parallelRadialDirection t := by
  simp only [parallelRadialParameterPoint, sub_smul]
  ring

/-- CFT-12-E04. -/
theorem exercise_04_solution (z w : ℂ) :
    Complex.orientation.areaForm z w = -Complex.orientation.areaForm w z ∧
      Complex.orientation.areaForm z z = 0 :=
  ⟨Complex.orientation.areaForm_swap z w,
    Complex.orientation.areaForm_apply_self z⟩

/-- CFT-12-E05. -/
theorem exercise_05_solution (z w : ℂ) :
    Complex.orientation.areaForm z (-w) = -Complex.orientation.areaForm z w ∧
      Complex.orientation.areaForm (-z) w = -Complex.orientation.areaForm z w := by
  constructor
  · simp
  · simp

/-- CFT-12-E06. -/
theorem exercise_06_solution {a rho : ℝ} {u n v : ℂ}
    (hv : v = ((a : ℂ) + Complex.I * (rho : ℂ)) * u)
    (hu : ‖u‖ = 1) (hrho : 0 < rho)
    (hn : ‖n‖ = 1) (horth : ⟪n, v⟫_ℝ = 0)
    (hforward : 0 < ⟪n, u⟫_ℝ) :
    0 < Complex.orientation.areaForm n v ∧
      v = Complex.I * n * (‖v‖ : ℂ) :=
  ⟨complexAreaForm_radialTangent_pos hv hu hrho hn horth hforward,
    radialTangent_eq_I_mul_normal_mul_norm hv hu hrho hn horth hforward⟩

end Exercises.Chapter12

end CrouzeixTextbook.Part02
