import CrouzeixConjecture.HolomorphicOuterLimit

namespace CrouzeixTextbook.Part03
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def holomorphic_matrix_eval := @holomorphicMatrixEval
set_option linter.defProp false in
def contour_eval_agrees := @PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_holomorphicMatrixEval
set_option linter.defProp false in
def polynomial_compatibility := @holomorphicMatrixEval_polynomial
set_option linter.defProp false in
def locality_on_neighborhood := @holomorphicMatrixEval_congr_on_neighborhood
set_option linter.defProp false in
def functional_calculus_additive := @holomorphicMatrixEval_add
set_option linter.defProp false in
def functional_calculus_multiplicative := @holomorphicMatrixEval_mul

namespace Exercises.Chapter16

open Filter Set
open scoped Topology

variable {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]

/-- CFT-16-E01. -/
theorem exercise_01_solution (A : SquareMatrix n) (f : ℂ → ℂ) :
    holomorphicMatrixEval A f =
      limUnder atTop (simpleSpectrumHolomorphicEval A f) := rfl

/-- CFT-16-E02. -/
theorem exercise_02_solution (A : SquareMatrix n) :
    holomorphicMatrixEval A (fun z => z) = A := by
  have h := holomorphicMatrixEval_polynomial A Polynomial.X
  simpa [polynomialEval] using h

/-- CFT-16-E03. -/
theorem exercise_03_solution
    (R₁ R₂ : PositivePeriodicRadialData) (c₁ c₂ : ℂ) {Ω₁ Ω₂ V₁ V₂ : Set ℂ}
    (G₁ : PositivePeriodicRadialData.OrientedRadialConvexBoundary R₁ c₁ Ω₁)
    (G₂ : PositivePeriodicRadialData.OrientedRadialConvexBoundary R₂ c₂ Ω₂)
    (h₁open : IsOpen V₁) (h₁convex : Convex ℝ V₁) (h₁closure : closure Ω₁ ⊆ V₁)
    (h₂open : IsOpen V₂) (h₂convex : Convex ℝ V₂) (h₂closure : closure Ω₂ ⊆ V₂)
    {f : ℂ → ℂ} (h₁f : DifferentiableOn ℂ f V₁) (h₂f : DifferentiableOn ℂ f V₂)
    (A : SquareMatrix n) (h₁WA : numericalRange A ⊆ Ω₁) (h₂WA : numericalRange A ⊆ Ω₂) :
    parametricBoundaryIntegral (G₁.parametricBoundary R₁ c₁) contourParameterMeasure f A =
      parametricBoundaryIntegral (G₂.parametricBoundary R₂ c₂) contourParameterMeasure f A := by
  rw [G₁.parametricBoundaryIntegral_eq_holomorphicMatrixEval
      R₁ c₁ h₁open h₁convex h₁closure h₁f A h₁WA,
    G₂.parametricBoundaryIntegral_eq_holomorphicMatrixEval
      R₂ c₂ h₂open h₂convex h₂closure h₂f A h₂WA]

open scoped Classical in
/-- CFT-16-E04. -/
theorem exercise_04_solution (A : SquareMatrix n) {U : Set ℂ} (hUopen : IsOpen U)
    (hWU : numericalRange A ⊆ U) (f : ℂ → ℂ) (c : ℂ) :
    holomorphicMatrixEval A f =
      holomorphicMatrixEval A (fun z => if z ∈ U then f z else c) := by
  refine holomorphicMatrixEval_congr_on_neighborhood A hUopen hWU ?_
  intro z hz
  rw [if_pos hz]

/-- CFT-16-E05. -/
theorem exercise_05_solution (A : SquareMatrix n) {U V : Set ℂ}
    (hUopen : IsOpen U) (hVopen : IsOpen V)
    (hWU : numericalRange A ⊆ U ∩ V) {f g : ℂ → ℂ}
    (hf : DifferentiableOn ℂ f U) (hg : DifferentiableOn ℂ g V) :
    holomorphicMatrixEval A (fun z => f z + g z) =
      holomorphicMatrixEval A f + holomorphicMatrixEval A g :=
  holomorphicMatrixEval_add A (hUopen.inter hVopen) hWU
    (hf.mono Set.inter_subset_left) (hg.mono Set.inter_subset_right)

/-- CFT-16-E06. -/
theorem exercise_06_solution (A : SquareMatrix n) {U : Set ℂ} (hUopen : IsOpen U)
    (hWU : numericalRange A ⊆ U) {f : ℂ → ℂ} (hf : DifferentiableOn ℂ f U) :
    holomorphicMatrixEval A (fun z => f z * f z) =
      holomorphicMatrixEval A f * holomorphicMatrixEval A f :=
  holomorphicMatrixEval_mul A hUopen hWU hf hf

end Exercises.Chapter16

end
end CrouzeixTextbook.Part03
