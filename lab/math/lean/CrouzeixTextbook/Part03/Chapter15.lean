import CrouzeixConjecture.HolomorphicFunctionalCalculus

namespace CrouzeixTextbook.Part03
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def parametric_boundary_integral := @parametricBoundaryIntegral
set_option linter.defProp false in
def boundary_integral_continuous := @continuousOn_parametricBoundaryIntegral
set_option linter.defProp false in
def boundary_integral_is_function_eval := @PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_functionEval
set_option linter.defProp false in
def simple_spectrum_holomorphic_eval := @simpleSpectrumHolomorphicEval
set_option linter.defProp false in
def simple_spectrum_boundary_limit := @tendsto_parametricBoundaryIntegral_simpleSpectrumApproximation
set_option linter.defProp false in
def simple_spectrum_eval_limit := @PositivePeriodicRadialData.OrientedRadialConvexBoundary.tendsto_simpleSpectrumHolomorphicEval

namespace Exercises.Chapter15

open Filter MeasureTheory Set
open scoped Matrix Matrix.Norms.L2Operator Topology

variable {i n : Type*} [TopologicalSpace i] [CompactSpace i]
  [MeasurableSpace i] [OpensMeasurableSpace i]
  [Fintype n] [DecidableEq n] [Nonempty n]
  {mu : Measure i} [IsFiniteMeasure mu] {Omega : Set ℂ}

omit [CompactSpace i] [OpensMeasurableSpace i] [Nonempty n] [IsFiniteMeasure mu] in
/-- CFT-15-E01. -/
theorem exercise_01_solution
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (f : ℂ → ℂ) (B : SquareMatrix n) :
    parametricBoundaryIntegral Gamma mu f B =
      ∫ x, f (Gamma.point x) • parametricBoundaryFirstPart Gamma B x ∂mu := rfl

omit [CompactSpace i] [OpensMeasurableSpace i] [Nonempty n] [IsFiniteMeasure mu] in
/-- CFT-15-E02. -/
theorem exercise_02_solution
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) :
    parametricBoundaryIntegral Gamma mu (fun _ => 0) B = 0 := by
  simp [parametricBoundaryIntegral]

/-- CFT-15-E03. -/
theorem exercise_03_solution
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (f : ℂ → ℂ) (hf : ContinuousOn f (closure Omega))
    (A : SquareMatrix n) (hOmegaOpen : IsOpen Omega)
    (hWA : numericalRange A ⊆ Omega) :
    ContinuousOn (parametricBoundaryIntegral Gamma mu f)
        {B : SquareMatrix n | numericalRange B ⊆ Omega} ∧
      Tendsto
        (fun k => parametricBoundaryIntegral Gamma mu f
          (simpleSpectrumApproximation A k)) atTop
        (nhds (parametricBoundaryIntegral Gamma mu f A)) :=
  ⟨continuousOn_parametricBoundaryIntegral Gamma f hf,
    tendsto_parametricBoundaryIntegral_simpleSpectrumApproximation
      Gamma f hf A hOmegaOpen hWA⟩

omit [Nonempty n] in
/-- CFT-15-E04. -/
theorem exercise_04_solution
    (R : PositivePeriodicRadialData) (c : ℂ) {V : Set ℂ}
    (G : PositivePeriodicRadialData.OrientedRadialConvexBoundary R c Omega)
    (hVopen : IsOpen V) (hVconvex : Convex ℝ V)
    (hclosure : closure Omega ⊆ V)
    {f : ℂ → ℂ} (hf : DifferentiableOn ℂ f V)
    (B : SquareMatrix n) (hB : SimpleDiagonalization B)
    (hWB : numericalRange B ⊆ Omega) :
    parametricBoundaryIntegral (G.parametricBoundary R c)
        contourParameterMeasure f B = hB.functionEval f :=
  G.parametricBoundaryIntegral_eq_functionEval R c hVopen hVconvex hclosure hf B hB hWB

/-- CFT-15-E05. -/
theorem exercise_05_solution (A : SquareMatrix n) (f : ℂ → ℂ) (k : ℕ) :
    simpleSpectrumHolomorphicEval A f k =
      (simpleDiagonalization_of_hasDistinctEigenvalues
        (simpleSpectrumApproximation A k)
        (simpleSpectrumApproximation_hasDistinctEigenvalues A k)).functionEval f := rfl

/-- CFT-15-E06. -/
theorem exercise_06_solution
    (R : PositivePeriodicRadialData) (c : ℂ) {V : Set ℂ}
    (G : PositivePeriodicRadialData.OrientedRadialConvexBoundary R c Omega)
    (hVopen : IsOpen V) (hVconvex : Convex ℝ V)
    (hclosure : closure Omega ⊆ V)
    {f : ℂ → ℂ} (hf : DifferentiableOn ℂ f V)
    (A : SquareMatrix n) (hWA : numericalRange A ⊆ Omega) :
    Tendsto (simpleSpectrumHolomorphicEval A f) atTop
      (nhds (parametricBoundaryIntegral (G.parametricBoundary R c)
        contourParameterMeasure f A)) :=
  G.tendsto_simpleSpectrumHolomorphicEval R c hVopen hVconvex hclosure hf A hWA

end Exercises.Chapter15

end
end CrouzeixTextbook.Part03
