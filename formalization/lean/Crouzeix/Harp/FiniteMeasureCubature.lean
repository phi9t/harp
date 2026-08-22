import Crouzeix.Harp.PositiveCubature
import Mathlib.MeasureTheory.Integral.Average

/-!
Positive cubature for nonzero finite measures.

The convex-geometric theorem in `PositiveCubature` is normalized for
probability measures.  This file supplies the exact rescaling needed by the
finite atomic dilation construction.
-/

noncomputable section

namespace CrouzeixConjecture
namespace Harp

open MeasureTheory Set
open scoped BigOperators

universe u v

/-- A nonzero finite measure has an exact strictly positive cubature
formula for every continuous finite-dimensional real observable. -/
theorem exists_positive_cubature_finite_measure
    {X : Type u} {E : Type v}
    [TopologicalSpace X] [MeasurableSpace X] [OpensMeasurableSpace X]
    [NormedAddCommGroup E] [NormedSpace ℝ E] [FiniteDimensional ℝ E]
    (μ : Measure X) [IsFiniteMeasure μ] [NeZero μ]
    {α : Set X} (hα_compact : IsCompact α) (hα_measurable : MeasurableSet α)
    (hμ_support : ∀ᵐ x ∂μ, x ∈ α)
    (g : X → E) (hg : ContinuousOn g α) :
    ∃ (ι : Type v) (_ : Fintype ι) (nodes : ι → X) (weights : ι → ℝ),
      Fintype.card ι ≤ Module.finrank ℝ E + 1 ∧
      (∀ j, nodes j ∈ α) ∧
      (∀ j, 0 < weights j) ∧
      ∑ j, weights j = μ.real Set.univ ∧
      ∑ j, weights j • g (nodes j) = ∫ x, g x ∂μ := by
  let ν : Measure X := (μ Set.univ)⁻¹ • μ
  haveI : IsProbabilityMeasure ν := inferInstance
  have hν_support : ∀ᵐ x ∂ν, x ∈ α :=
    Measure.ae_smul_measure hμ_support (μ Set.univ)⁻¹
  obtain ⟨ι, hι, nodes, weights, hcard, hnodes, hweights_pos,
      hweights_sum, hweighted_sum⟩ :=
    exists_positive_cubature ν hα_compact hα_measurable hν_support g hg
  let mass : ℝ := μ.real Set.univ
  refine ⟨ι, hι, nodes, fun j => mass * weights j, hcard, hnodes, ?_, ?_, ?_⟩
  · intro j
    exact mul_pos measureReal_univ_pos (hweights_pos j)
  · simp [mass, ← Finset.mul_sum, hweights_sum]
  · calc
      ∑ j, (mass * weights j) • g (nodes j) =
          mass • ∑ j, weights j • g (nodes j) := by
            simp only [mul_smul, Finset.smul_sum]
      _ = mass • ∫ x, g x ∂ν := by rw [hweighted_sum]
      _ = μ.real Set.univ • ⨍ x, g x ∂μ := by
        rfl
      _ = ∫ x, g x ∂μ := measure_smul_average μ g

end Harp
end CrouzeixConjecture
