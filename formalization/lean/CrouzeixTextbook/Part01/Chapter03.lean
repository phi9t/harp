import Mathlib.LinearAlgebra.Dimension.Finite
import Mathlib.LinearAlgebra.FiniteDimensional.Lemmas

namespace CrouzeixTextbook.Part01

/-- CFT-03-001: kernel membership is the zero-output equation. -/
theorem mem_kernel_iff {𝕜 V W : Type*} [Semiring 𝕜]
    [AddCommMonoid V] [Module 𝕜 V] [AddCommMonoid W] [Module 𝕜 W]
    (T : V →ₗ[𝕜] W) (x : V) : x ∈ T.ker ↔ T x = 0 :=
  LinearMap.mem_ker

/-- CFT-03-002: range membership is existence of a preimage. -/
theorem mem_range_iff {𝕜 V W : Type*} [Semiring 𝕜]
    [AddCommMonoid V] [Module 𝕜 V] [AddCommMonoid W] [Module 𝕜 W]
    (T : V →ₗ[𝕜] W) (y : W) : y ∈ T.range ↔ ∃ x, T x = y :=
  LinearMap.mem_range

/-- CFT-03-003: finite-dimensional rank plus nullity equals dimension. -/
theorem rank_nullity {𝕜 V W : Type*} [DivisionRing 𝕜]
    [AddCommGroup V] [Module 𝕜 V] [AddCommGroup W] [Module 𝕜 W]
    [Module.Finite 𝕜 V] (T : V →ₗ[𝕜] W) :
    Module.finrank 𝕜 T.range + Module.finrank 𝕜 T.ker = Module.finrank 𝕜 V :=
  LinearMap.finrank_range_add_finrank_ker T

/-- CFT-03-004: injectivity is exactly a trivial kernel. -/
theorem injective_iff_kernel_bottom {𝕜 V W : Type*} [DivisionRing 𝕜]
    [AddCommGroup V] [Module 𝕜 V] [AddCommGroup W] [Module 𝕜 W]
    (T : V →ₗ[𝕜] W) : Function.Injective T ↔ T.ker = ⊥ := by
  simpa [eq_comm] using (LinearMap.ker_eq_bot (f := T)).symm

/-- CFT-03-005: a map restricts to any invariant subspace. -/
def invariantRestriction {𝕜 V : Type*} [Semiring 𝕜] [AddCommMonoid V]
    [Module 𝕜 V] (T : V →ₗ[𝕜] V) (W : Submodule 𝕜 V)
    (hW : ∀ x ∈ W, T x ∈ W) : W →ₗ[𝕜] W :=
  { toFun := fun x => ⟨T x, hW x x.property⟩
    map_add' := fun x y => Subtype.ext (T.map_add x y)
    map_smul' := fun c x => Subtype.ext (T.map_smul c x) }

/-- CFT-03-006: composition cannot have range larger than its final map. -/
theorem composition_range_le {𝕜 U V W : Type*} [Semiring 𝕜]
    [AddCommMonoid U] [Module 𝕜 U] [AddCommMonoid V] [Module 𝕜 V]
    [AddCommMonoid W] [Module 𝕜 W] (S : U →ₗ[𝕜] V) (T : V →ₗ[𝕜] W) :
    LinearMap.range (T.comp S) ≤ LinearMap.range T :=
  LinearMap.range_comp_le_range S T

end CrouzeixTextbook.Part01
