import CrouzeixTextbook.Part01.Chapter01
import Mathlib.LinearAlgebra.Dual.Lemmas
import Mathlib.LinearAlgebra.Matrix.Trace

namespace CrouzeixTextbook.Part01

/-- CFT-04-001: pullback of a functional is evaluation after the map. -/
theorem dual_map_apply {𝕜 V W : Type*} [CommSemiring 𝕜]
    [AddCommMonoid V] [Module 𝕜 V] [AddCommMonoid W] [Module 𝕜 W]
    (T : V →ₗ[𝕜] W) (φ : Module.Dual 𝕜 W) (x : V) :
    T.dualMap φ x = φ (T x) :=
  rfl

/-- CFT-04-002: dualization reverses composition. -/
theorem dual_map_composition {𝕜 U V W : Type*} [CommSemiring 𝕜]
    [AddCommMonoid U] [Module 𝕜 U] [AddCommMonoid V] [Module 𝕜 V]
    [AddCommMonoid W] [Module 𝕜 W] (S : U →ₗ[𝕜] V) (T : V →ₗ[𝕜] W) :
    S.dualMap.comp T.dualMap = (T.comp S).dualMap :=
  LinearMap.dualMap_comp_dualMap S T

/-- CFT-04-003: coordinate change conjugates the matrix action. -/
def coordinate_change_conjugacy := @change_basis_action

/-- CFT-04-004: characteristic polynomial is similarity-invariant. -/
def duality_similarity_preserves_charpoly := @similarity_preserves_charpoly

/-- CFT-04-005: determinant is similarity-invariant. -/
theorem similarity_preserves_determinant {𝕜 n : Type*} [CommRing 𝕜]
    [Fintype n] [DecidableEq n] (S : (Matrix n n 𝕜)ˣ) (A : Matrix n n 𝕜) :
    (S.val * A * (↑(S⁻¹) : Matrix n n 𝕜)).det = A.det :=
  Matrix.det_units_conj S A

/-- CFT-04-006: trace is similarity-invariant. -/
theorem similarity_preserves_trace {𝕜 n : Type*} [CommSemiring 𝕜]
    [Fintype n] [DecidableEq n] (S : (Matrix n n 𝕜)ˣ) (A : Matrix n n 𝕜) :
    (S.val * A * (↑(S⁻¹) : Matrix n n 𝕜)).trace = A.trace :=
  Matrix.trace_units_conj S A

end CrouzeixTextbook.Part01
