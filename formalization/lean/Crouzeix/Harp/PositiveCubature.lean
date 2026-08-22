import Mathlib.Analysis.Convex.Caratheodory
import Mathlib.Analysis.Convex.Integral
import Mathlib.Analysis.Convex.Topology
import Mathlib.Analysis.Normed.Module.FiniteDimension
import Mathlib.LinearAlgebra.AffineSpace.FiniteDimensional
import Mathlib.MeasureTheory.SpecificCodomains.Pi

/-!
Finite-dimensional compact convex hulls and exact positive cubature.

This is the Harp-owned foundation for replacing a probability integral of a
continuous finite-dimensional observable by an exact finite convex
combination of values of that observable.
-/

noncomputable section

namespace CrouzeixConjecture
namespace Harp

open MeasureTheory Set
open scoped BigOperators

universe u v

/-- The convex hull of a compact subset of a finite-dimensional real normed
space is compact. -/
theorem isCompact_convexHull_real
    {E : Type*} [NormedAddCommGroup E] [NormedSpace ℝ E]
    [FiniteDimensional ℝ E] {s : Set E} (hs : IsCompact s) :
    IsCompact (convexHull ℝ s) := by
  let N := Module.finrank ℝ E + 1
  let bary (n : ℕ) : (Fin n → ℝ) × (Fin n → E) → E :=
    fun p => ∑ i, p.1 i • p.2 i
  let combinations (n : ℕ) : Set E :=
    bary n ''
      (stdSimplex ℝ (Fin n) ×ˢ Set.pi Set.univ (fun _ : Fin n => s))
  have hbary (n : ℕ) : Continuous (bary n) := by
    dsimp [bary]
    fun_prop
  have hcombinations (n : ℕ) : IsCompact (combinations n) := by
    apply IsCompact.image
    · exact (isCompact_stdSimplex ℝ (Fin n)).prod
        (isCompact_univ_pi fun _ => hs)
    · exact hbary n
  have hrepresentation :
      convexHull ℝ s = ⋃ n ∈ Finset.range (N + 1), combinations n := by
    ext x
    constructor
    · intro hx
      let t := Caratheodory.minCardFinsetOfMemConvexHull hx
      have ht_subset : (t : Set E) ⊆ s :=
        Caratheodory.minCardFinsetOfMemConvexHull_subseteq hx
      have ht_independent : AffineIndependent ℝ ((↑) : t → E) :=
        Caratheodory.affineIndependent_minCardFinsetOfMemConvexHull hx
      have hx_t : x ∈ convexHull ℝ (t : Set E) :=
        Caratheodory.mem_minCardFinsetOfMemConvexHull hx
      have ht_card : t.card ≤ N := by
        calc
          t.card = Fintype.card t := by simp
          _ ≤ Module.finrank ℝ (vectorSpan ℝ (Set.range ((↑) : t → E))) + 1 :=
            ht_independent.card_le_finrank_succ
          _ ≤ Module.finrank ℝ E + 1 :=
            Nat.add_le_add_right (Submodule.finrank_le _) 1
          _ = N := rfl
      obtain ⟨w, hw_nonneg, hw_sum, hw_bary⟩ :=
        (Finset.mem_convexHull' (R := ℝ)).mp hx_t
      let e : t ≃ Fin t.card :=
        (Fintype.equivFin t).trans (finCongr (by simp))
      let W : Fin t.card → ℝ := fun j => w (e.symm j)
      let Z : Fin t.card → E := fun j => e.symm j
      have hW_nonneg : ∀ j, 0 ≤ W j := by
        intro j
        exact hw_nonneg _ (e.symm j).property
      have hW_sum : ∑ j, W j = 1 := by
        calc
          ∑ j, W j = ∑ i : t, w i := by
            simpa [W] using Equiv.sum_comp e.symm (fun i : t => w i)
          _ = ∑ y ∈ t, w y := by
            exact Finset.sum_attach t w
          _ = 1 := hw_sum
      have hW_bary : ∑ j, W j • Z j = x := by
        calc
          ∑ j, W j • Z j = ∑ i : t, w i • (i : E) := by
            simpa [W, Z] using
              Equiv.sum_comp e.symm (fun i : t => w i • (i : E))
          _ = ∑ y ∈ t, w y • y := by
            exact Finset.sum_attach t (fun y => w y • y)
          _ = x := hw_bary
      apply Set.mem_iUnion.mpr
      refine ⟨t.card, Set.mem_iUnion.mpr ⟨?_, ?_⟩⟩
      · exact Finset.mem_range.mpr (Nat.lt_succ_of_le ht_card)
      · refine ⟨(W, Z), ?_, ?_⟩
        · exact ⟨⟨hW_nonneg, hW_sum⟩,
            Set.mem_univ_pi.mpr fun j => ht_subset (e.symm j).property⟩
        · simpa [bary] using hW_bary
    · intro hx
      rcases Set.mem_iUnion.mp hx with ⟨n, hx⟩
      rcases Set.mem_iUnion.mp hx with ⟨_hn, hx⟩
      rcases hx with ⟨p, hp, rfl⟩
      exact mem_convexHull_of_exists_fintype p.1 p.2 hp.1.1 hp.1.2
        (Set.mem_univ_pi.mp hp.2) rfl
  rw [hrepresentation]
  exact (Finset.range (N + 1)).isCompact_biUnion fun n _ => hcombinations n

/-- A probability integral of a continuous finite-dimensional observable is
an exact strictly positive finite cubature formula supported on the compact
carrier.  The number of nodes is at most one more than the real dimension of
the observable space. -/
theorem exists_positive_cubature
    {X : Type u} {E : Type v}
    [TopologicalSpace X] [MeasurableSpace X] [OpensMeasurableSpace X]
    [NormedAddCommGroup E] [NormedSpace ℝ E] [FiniteDimensional ℝ E]
    (μ : Measure X) [IsProbabilityMeasure μ]
    {α : Set X} (hα_compact : IsCompact α) (hα_measurable : MeasurableSet α)
    (hμ_support : ∀ᵐ x ∂μ, x ∈ α)
    (g : X → E) (hg : ContinuousOn g α) :
    ∃ (ι : Type v) (_ : Fintype ι) (nodes : ι → X) (weights : ι → ℝ),
      Fintype.card ι ≤ Module.finrank ℝ E + 1 ∧
      (∀ i, nodes i ∈ α) ∧
      (∀ i, 0 < weights i) ∧
      ∑ i, weights i = 1 ∧
      ∑ i, weights i • g (nodes i) = ∫ x, g x ∂μ := by
  have hg_integrable_on : IntegrableOn g α μ :=
    hg.integrableOn_of_subset_isCompact hα_compact hα_measurable Subset.rfl
      (measure_ne_top μ α)
  have hrestrict : μ.restrict α = μ :=
    Measure.restrict_eq_self_of_ae_mem hμ_support
  have hg_integrable : Integrable g μ := by
    rw [← hrestrict]
    exact hg_integrable_on
  have himage_compact : IsCompact (g '' α) :=
    hα_compact.image_of_continuousOn hg
  have hintegral_hull : (∫ x, g x ∂μ) ∈ convexHull ℝ (g '' α) := by
    exact (convex_convexHull ℝ (g '' α)).integral_mem
      (isCompact_convexHull_real himage_compact).isClosed
      (hμ_support.mono fun x hx => subset_convexHull ℝ _ ⟨x, hx, rfl⟩)
      hg_integrable
  obtain ⟨ι, hι, z, weights, hz, hz_independent, hweights_pos,
      hweights_sum, hweighted_sum⟩ :=
    eq_pos_convex_span_of_mem_convexHull hintegral_hull
  let _ : Fintype ι := hι
  choose nodes hnodes_value hnodes_mem using fun i => hz (Set.mem_range_self i)
  refine ⟨ι, hι, nodes, weights, ?_, hnodes_value, hweights_pos,
    hweights_sum, ?_⟩
  · calc
      Fintype.card ι ≤ Module.finrank ℝ (vectorSpan ℝ (Set.range z)) + 1 :=
        hz_independent.card_le_finrank_succ
      _ ≤ Module.finrank ℝ E + 1 :=
        Nat.add_le_add_right (Submodule.finrank_le _) 1
  · calc
      ∑ i, weights i • g (nodes i) = ∑ i, weights i • z i := by
        apply Finset.sum_congr rfl
        intro i _
        rw [hnodes_mem i]
      _ = ∫ x, g x ∂μ := hweighted_sum

/-- Exact positive cubature for every real-valued moment indexed from zero
through `N + 1`.  The common formula uses at most `N + 3` nodes. -/
theorem exists_positive_cubature_moments_through_succ
    {X : Type u}
    [TopologicalSpace X] [MeasurableSpace X] [OpensMeasurableSpace X]
    (N : ℕ) (μ : Measure X) [IsProbabilityMeasure μ]
    {α : Set X} (hα_compact : IsCompact α) (hα_measurable : MeasurableSet α)
    (hμ_support : ∀ᵐ x ∂μ, x ∈ α)
    (φ : Fin (N + 2) → X → ℝ) (hφ : ∀ k, ContinuousOn (φ k) α) :
    ∃ (ι : Type) (_ : Fintype ι) (nodes : ι → X) (weights : ι → ℝ),
      Fintype.card ι ≤ N + 3 ∧
      (∀ i, nodes i ∈ α) ∧
      (∀ i, 0 < weights i) ∧
      ∑ i, weights i = 1 ∧
      ∀ k, ∑ i, weights i * φ k (nodes i) = ∫ x, φ k x ∂μ := by
  let g : X → (Fin (N + 2) → ℝ) := fun x k => φ k x
  have hg : ContinuousOn g α := by
    exact continuousOn_pi.mpr hφ
  obtain ⟨ι, hι, nodes, weights, hcard, hnodes, hweights_pos,
      hweights_sum, hweighted_sum⟩ :=
    exists_positive_cubature μ hα_compact hα_measurable hμ_support g hg
  have hφ_integrable : ∀ k, Integrable (φ k) μ := by
    intro k
    have hφ_integrable_on : IntegrableOn (φ k) α μ :=
      (hφ k).integrableOn_of_subset_isCompact hα_compact hα_measurable
        Subset.rfl (measure_ne_top μ α)
    rw [← Measure.restrict_eq_self_of_ae_mem hμ_support]
    exact hφ_integrable_on
  refine ⟨ι, hι, nodes, weights, ?_, hnodes, hweights_pos,
    hweights_sum, ?_⟩
  · simpa [Module.finrank_pi, Nat.add_assoc] using hcard
  · intro k
    calc
      ∑ i, weights i * φ k (nodes i) =
          (∑ i, weights i • g (nodes i)) k := by simp [g]
      _ = (∫ x, g x ∂μ) k := congrFun hweighted_sum k
      _ = ∫ x, φ k x ∂μ := by
        simpa [g] using MeasureTheory.eval_integral hφ_integrable k

end Harp
end CrouzeixConjecture
