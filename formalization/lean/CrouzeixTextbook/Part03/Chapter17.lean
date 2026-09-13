import CrouzeixConjecture.RationalFunctionalCalculus

namespace CrouzeixTextbook.Part03
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def rational_pole_set := @rationalPoleSet
set_option linter.defProp false in
def rational_poles_finite := @rationalPoleSet_finite
set_option linter.defProp false in
def pole_complement_open := @isOpen_compl_rationalPoleSet
set_option linter.defProp false in
def rational_pole_free_on := @RationalPoleFreeOn
set_option linter.defProp false in
def rational_scalar_eval := @rationalScalarEval
set_option linter.defProp false in
def rational_matrix_eval := @rationalMatrixEval

namespace Exercises.Chapter17

open Set

variable {n : Type*} [Fintype n] [DecidableEq n]

/-- CFT-17-E01. -/
theorem exercise_01_solution (r : RatFunc ℂ) :
    rationalPoleSet r = {z | Polynomial.eval z r.denom = 0} := rfl

/-- CFT-17-E02. -/
theorem exercise_02_solution (p : Polynomial ℂ) (A : SquareMatrix n) :
    rationalMatrixEval (algebraMap (Polynomial ℂ) (RatFunc ℂ) p) A = polynomialEval p A := by
  unfold rationalMatrixEval
  rw [RatFunc.num_algebraMap, RatFunc.denom_algebraMap]
  have hone : polynomialEval (1 : Polynomial ℂ) A = 1 := by
    simp [polynomialEval]
  rw [hone, inv_one, mul_one]

/-- CFT-17-E03. -/
theorem exercise_03_solution (r : RatFunc ℂ) (s : Set ℂ) (hfree : RationalPoleFreeOn r s) :
    ∃ U : Set ℂ, IsOpen U ∧ s ⊆ U ∧ RationalPoleFreeOn r U :=
  ⟨(rationalPoleSet r)ᶜ, isOpen_compl_rationalPoleSet r,
    (rationalPoleFreeOn_iff_subset_compl r s).mp hfree,
    rationalPoleFreeOn_compl_rationalPoleSet r⟩

/-- CFT-17-E04. -/
theorem exercise_04_solution (r : RatFunc ℂ) {s t : Set ℂ} (hst : s ⊆ t)
    (hfree : RationalPoleFreeOn r t) : RationalPoleFreeOn r s :=
  Disjoint.mono_right hst hfree

/-- CFT-17-E05. -/
theorem exercise_05_solution (r : RatFunc ℂ) {z : ℂ} (hz : z ∈ rationalPoleSet r) :
    rationalScalarEval r z = 0 := by
  have hzero : Polynomial.eval z r.denom = 0 := hz
  unfold rationalScalarEval
  rw [hzero, div_zero]

/-- CFT-17-E06. -/
theorem exercise_06_solution [Nonempty n] (r : RatFunc ℂ) (A : SquareMatrix n)
    (hfree : RationalPoleFreeOn r (numericalRange A)) :
    rationalMatrixEval r A * polynomialEval r.denom A = polynomialEval r.num A := by
  have hunit := polynomialEval_denom_isUnit_of_rationalPoleFreeOn_numericalRange r A hfree
  rw [Matrix.isUnit_iff_isUnit_det] at hunit
  unfold rationalMatrixEval
  exact Matrix.nonsing_inv_mul_cancel_right _ _ hunit

end Exercises.Chapter17

end
end CrouzeixTextbook.Part03
