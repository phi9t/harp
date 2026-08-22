module

public import CrouzeixConjecture.ParametricBoundary
public import CrouzeixConjecture.ParametricDoubleLayerIdentity
public import CrouzeixConjecture.DoubleLayerCayley
public import CrouzeixConjecture.MainPerturbationReduction

@[expose] public section

noncomputable section

open MeasureTheory
open scoped BoundedContinuousFunction ComplexConjugate ComplexOrder Matrix
  Matrix.Norms.L2Operator

namespace CrouzeixConjecture

variable {i n : Type*} [TopologicalSpace i] [CompactSpace i]
  [MeasurableSpace i] [OpensMeasurableSpace i]
  [Fintype n] [DecidableEq n] [Nonempty n]

/-- The manuscript's unit bound on the closed domain restricts to a contractive boundary
function because every parametrized point lies in the frontier, hence in the closure. -/
def parametricContractivePolynomialBoundaryFunction
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (q : Polynomial ℂ)
    (hq : ∀ z ∈ closure Omega, ‖Polynomial.eval z q‖ ≤ 1) :
    ContractiveBoundaryFunction i where
  function := parametricPolynomialBoundaryFunction Gamma q
  norm_le_one x := hq (Gamma.point x)
    (frontier_subset_closure (Gamma.supported x).boundary_point)

/-- The polynomial Cauchy formula extends to the Cayley boundary function by uniform convergence. -/
theorem parametricBoundaryFirstPartIntegral_cayley_eq
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (hCauchy : HasParametricPolynomialCauchyFormula Gamma mu B)
    (q : Polynomial ℂ)
    (hq : ∀ w ∈ closure Omega, ‖Polynomial.eval w q‖ ≤ 1)
    (z : ℂ) (hz : z ∈ unitDisk) :
    (∫ x, cayleyBoundaryFunction z
          (by simpa [unitDisk, Metric.mem_ball, dist_eq_norm] using hz)
          (parametricContractivePolynomialBoundaryFunction Gamma q hq) x •
        parametricBoundaryFirstPart Gamma B x ∂mu) =
      matrixCayleyTransform z (polynomialEval q B) := by
  let f := parametricContractivePolynomialBoundaryFunction Gamma q hq
  let L := parametricBoundaryFirstPartIntegralCLM (mu := mu) Gamma B hWB
  have hznorm : ‖z‖ < 1 := by
    simpa [unitDisk, Metric.mem_ball, dist_eq_norm] using hz
  have hmapped := (cayleyBoundarySeriesTerm_hasSum z hznorm f).mapL L
  have hspectrumB : matrixSpectrum B ⊆ closure Omega := by
    intro w hw
    exact subset_closure (hWB (matrixSpectrum_subset_numericalRange B hw))
  have hspectrumT : matrixSpectrum (polynomialEval q B) ⊆ closedUnitDisk :=
    matrixSpectrum_polynomialEval_subset_closedUnitDisk B q (closure Omega) hspectrumB hq
  have hmatrix := matrixCayleySeriesTerm_hasSum (polynomialEval q B) hspectrumT hz
  have heq :
      (fun m ↦ L (cayleyBoundarySeriesTerm z f m)) =
        matrixCayleySeriesTerm z (polynomialEval q B) := by
    funext m
    cases m with
    | zero =>
        change (∫ x, (1 : ℂ) • parametricBoundaryFirstPart Gamma B x ∂mu) = 1
        simpa [polynomialEval] using hCauchy (1 : Polynomial ℂ)
    | succ m =>
        have hboundaryPower :
            f.function ^ (m + 1) =
              parametricPolynomialBoundaryFunction Gamma (q ^ (m + 1)) := by
          ext x
          simp [f, parametricContractivePolynomialBoundaryFunction,
            parametricPolynomialBoundaryFunction, Polynomial.eval_pow]
        change L ((2 * z ^ (m + 1)) • (f.function ^ (m + 1))) =
          (2 * z ^ (m + 1)) • polynomialEval q B ^ (m + 1)
        rw [map_smul, hboundaryPower]
        change (2 * z ^ (m + 1)) •
            (∫ x, Polynomial.eval (Gamma.point x) (q ^ (m + 1)) •
              parametricBoundaryFirstPart Gamma B x ∂mu) = _
        rw [hCauchy (q ^ (m + 1))]
        simp [polynomialEval]
  rw [heq] at hmapped
  exact hmapped.unique hmatrix

/-- The companion associated with the full Cayley boundary function. -/
noncomputable def parametricCayleyCompanion
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (mu : Measure i) (B : SquareMatrix n)
    (q : Polynomial ℂ)
    (hq : ∀ w ∈ closure Omega, ‖Polynomial.eval w q‖ ≤ 1)
    (z : ℂ) : SquareMatrix n := by
  classical
  exact if hz : z ∈ unitDisk then
    parametricBoundaryCompanion Gamma mu B
      (cayleyBoundaryFunction z
        (by simpa [unitDisk, Metric.mem_ball, dist_eq_norm] using hz)
        (parametricContractivePolynomialBoundaryFunction Gamma q hq))
  else 0

omit [Nonempty n] in
theorem parametricCayleyCompanion_mem_generatedAlgebra
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (q : Polynomial ℂ)
    (hq : ∀ w ∈ closure Omega, ‖Polynomial.eval w q‖ ≤ 1)
    (z : ℂ) (hz : z ∈ unitDisk) :
    parametricCayleyCompanion Gamma mu B q hq z ∈ generatedAlgebra B := by
  rw [parametricCayleyCompanion, dif_pos hz]
  exact parametricBoundaryCompanion_mem_generatedAlgebra Gamma B hWB _

/-- The full double-layer identity in direct Cayley form. -/
theorem parametric_direct_cayley_identity
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (hCauchy : HasParametricPolynomialCauchyFormula Gamma mu B)
    (q : Polynomial ℂ)
    (hq : ∀ w ∈ closure Omega, ‖Polynomial.eval w q‖ ≤ 1)
    (z : ℂ) (hz : z ∈ unitDisk) :
    (2 : ℂ) • doubleLayerCayleySeries
        (parametricPositiveBoundaryDensity Gamma B hWB hCauchy)
        (parametricContractivePolynomialBoundaryFunction Gamma q hq) z =
      matrixCayleyTransform z (polynomialEval q B) +
        (parametricCayleyCompanion Gamma mu B q hq z)ᴴ := by
  have hznorm : ‖z‖ < 1 := by
    simpa [unitDisk, Metric.mem_ball, dist_eq_norm] using hz
  rw [doubleLayerCayleySeries_eq_value
    (parametricPositiveBoundaryDensity Gamma B hWB hCauchy)
    (parametricContractivePolynomialBoundaryFunction Gamma q hq) z hznorm]
  rw [parametricCayleyCompanion, dif_pos hz]
  exact two_smul_boundaryPhi_parametric_eq Gamma B hWB hCauchy _ _
    (parametricBoundaryFirstPartIntegral_cayley_eq Gamma B hWB hCauchy q hq z hz)

/-- A supported compact parametrization satisfying the explicit polynomial Cauchy integral
constructs the direct Cayley completion on the closed domain. -/
theorem hasDoubleLayerCompletionProvider_of_parametricBoundary
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (hCauchy : HasParametricPolynomialCauchyFormula Gamma mu B) :
    HasDoubleLayerCompletionProvider B (closure Omega) := by
  intro q hq
  let D : PositiveBoundaryDensity (n := n) mu :=
    parametricPositiveBoundaryDensity Gamma B hWB hCauchy
  let f : ContractiveBoundaryFunction i :=
    parametricContractivePolynomialBoundaryFunction Gamma q hq
  let g : ℂ → SquareMatrix n :=
    parametricCayleyCompanion Gamma mu B q hq
  refine ⟨doubleLayerCayleySeries D f, ?_⟩
  have hspectrumB : matrixSpectrum B ⊆ closure Omega := by
    intro z hz
    exact subset_closure (hWB (matrixSpectrum_subset_numericalRange B hz))
  apply isPositiveRealCompletion_of_direct_cayley_identity
    B (polynomialEval q B) (doubleLayerCayleySeries D f) g
  · exact matrixSpectrum_polynomialEval_subset_closedUnitDisk
      B q (closure Omega) hspectrumB hq
  · exact doubleLayerCayleySeries_analyticOnNhd D f
  · exact doubleLayerCayleySeries_zero D f
  · exact doubleLayerCayleySeries_rePart_posSemidef D f
  · intro z hz
    exact parametricCayleyCompanion_mem_generatedAlgebra Gamma B hWB q hq z hz
  · intro z hz
    exact parametric_direct_cayley_identity Gamma B hWB hCauchy q hq z hz

end CrouzeixConjecture
