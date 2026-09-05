import Mathlib.LinearAlgebra.Dimension.Finite
import Mathlib.LinearAlgebra.Basis.Defs
import Mathlib.LinearAlgebra.Quotient.Basic

namespace CrouzeixTextbook.Part01

/-- CFT-02-001: the span is the least subspace containing the generators. -/
theorem span_minimality {𝕜 V : Type*} [Semiring 𝕜] [AddCommMonoid V]
    [Module 𝕜 V] {s : Set V} {W : Submodule 𝕜 V} (h : s ⊆ W) :
    Submodule.span 𝕜 s ≤ W :=
  Submodule.span_le.mpr h

/-- CFT-02-002: intersection membership means satisfying both constraints. -/
theorem mem_subspace_intersection_iff {𝕜 V : Type*} [Semiring 𝕜]
    [AddCommMonoid V] [Module 𝕜 V] (U W : Submodule 𝕜 V) (x : V) :
    x ∈ U ⊓ W ↔ x ∈ U ∧ x ∈ W :=
  Submodule.mem_inf

/-- CFT-02-003: a basis gives unique coordinates. -/
theorem basis_coordinates_unique {ι 𝕜 V : Type*} [Semiring 𝕜]
    [AddCommMonoid V] [Module 𝕜 V] (b : Module.Basis ι 𝕜 V) {x y : V} :
    b.repr x = b.repr y ↔ x = y :=
  b.repr.injective.eq_iff

/-- CFT-02-004: linearly equivalent finite spaces have the same dimension. -/
theorem dimension_invariant_under_linear_equiv {𝕜 V W : Type*}
    [DivisionRing 𝕜] [AddCommGroup V] [Module 𝕜 V]
    [AddCommGroup W] [Module 𝕜 W] (e : V ≃ₗ[𝕜] W) :
    Module.finrank 𝕜 V = Module.finrank 𝕜 W :=
  e.finrank_eq

/-- CFT-02-005: a trivial intersection makes a two-summand decomposition unique. -/
theorem direct_sum_coordinates_unique {𝕜 V : Type*} [DivisionRing 𝕜]
    [AddCommGroup V] [Module 𝕜 V] (U W : Submodule 𝕜 V)
    (hdisjoint : U ⊓ W = ⊥) {u₁ u₂ w₁ w₂ : V}
    (hu₁ : u₁ ∈ U) (hu₂ : u₂ ∈ U) (hw₁ : w₁ ∈ W) (hw₂ : w₂ ∈ W)
    (hsum : u₁ + w₁ = u₂ + w₂) : u₁ = u₂ ∧ w₁ = w₂ := by
  have hdiff : u₁ - u₂ = w₂ - w₁ := by
    calc
      u₁ - u₂ = (u₁ + w₁) - (u₂ + w₁) := by abel
      _ = (u₂ + w₂) - (u₂ + w₁) := by rw [hsum]
      _ = w₂ - w₁ := by abel
  have hmem : u₁ - u₂ ∈ U ⊓ W := Submodule.mem_inf.mpr
    ⟨U.sub_mem hu₁ hu₂, hdiff ▸ W.sub_mem hw₂ hw₁⟩
  have hzero : u₁ - u₂ = 0 := by
    rw [hdisjoint] at hmem
    exact hmem
  constructor
  · exact sub_eq_zero.mp hzero
  · exact (sub_eq_zero.mp (hdiff ▸ hzero)).symm

/-- CFT-02-006: every quotient class has a representative. -/
theorem quotient_projection_surjective {𝕜 V : Type*} [Ring 𝕜]
    [AddCommGroup V] [Module 𝕜 V] (W : Submodule 𝕜 V) :
    Function.Surjective W.mkQ :=
  Submodule.mkQ_surjective W

end CrouzeixTextbook.Part01
