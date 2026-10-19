import CrouzeixConjecture.ParametricDoubleLayerIdentity
import CrouzeixConjecture.Spectrum

/-!
Algebraic facts for the Lorist--Schwenninger companion construction.

The conjugated boundary companion is a Bochner integral of elements of the
singly generated algebra `alg(B)`. Consequently it commutes with any target in
that algebra.
-/

noncomputable section

open MeasureTheory
open scoped BoundedContinuousFunction Matrix Matrix.Norms.L2Operator

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {i n : Type*} [TopologicalSpace i] [CompactSpace i]
  [MeasurableSpace i] [OpensMeasurableSpace i]
  [Fintype n] [DecidableEq n]
  {mu : Measure i} [IsFiniteMeasure mu] {Omega : Set ℂ}

omit [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] in
/-- The analytic half of the boundary density is pointwise in `alg(B)`. -/
theorem parametricBoundaryFirstPart_mem_generatedAlgebra
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega) (x : i) :
    parametricBoundaryFirstPart Gamma B x ∈ generatedAlgebra B := by
  have hres : parametricBoundaryResolvent Gamma B x ∈ generatedAlgebra B := by
    apply resolvent_mem_generatedAlgebra_of_numericalRange_subset B hWB
    exact (Gamma.supported x).sigma_not_mem
  have hnormal :
      Gamma.normal x • parametricBoundaryResolvent Gamma B x ∈
        generatedAlgebra B :=
    (generatedAlgebra B).smul_mem hres (Gamma.normal x)
  exact ((generatedAlgebra B).toSubmodule.restrictScalars ℝ).smul_mem
    ((2 * Real.pi : ℝ)⁻¹ * Gamma.speed x) hnormal

/-- The companion is bounded by the uniform boundary norm times the `L¹`
norm of the analytic half of the density. -/
theorem parametricBoundaryCompanion_norm_le
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (h : i →ᵇ ℂ) :
    ‖parametricBoundaryCompanion Gamma mu B h‖ ≤
      ‖h‖ * ∫ x, ‖parametricBoundaryFirstPart Gamma B x‖ ∂mu := by
  simpa only [parametricBoundaryCompanion,
      BoundedContinuousFunction.star_apply, norm_star, mul_comm] using
    (parametricBoundaryFirstPartIntegral_norm_le
      (mu := mu) Gamma B hWB (star h))

/-- A companion commutes with any target in the same singly generated algebra. -/
theorem parametricBoundaryCompanion_commute_of_mem_generatedAlgebra
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B T : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (h : i →ᵇ ℂ) (hT : T ∈ generatedAlgebra B) :
    Commute (parametricBoundaryCompanion Gamma mu B h) T := by
  have hcompanion :
      parametricBoundaryCompanion Gamma mu B h ∈ generatedAlgebra B :=
    CrouzeixConjecture.parametricBoundaryCompanion_mem_generatedAlgebra
      Gamma B hWB h
  obtain ⟨p, hp⟩ :=
    (generatedAlgebra_mem_iff_exists_polynomial B
      (parametricBoundaryCompanion Gamma mu B h)).mp hcompanion
  obtain ⟨q, hq⟩ :=
    (generatedAlgebra_mem_iff_exists_polynomial B T).mp hT
  rw [← hp, ← hq, commute_iff_eq]
  simp only [polynomialEval, ← map_mul]
  rw [mul_comm]

end LoristSchwenninger
end CrouzeixConjecture
