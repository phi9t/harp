import CrouzeixTextbook.Part01.Chapter01
import CrouzeixTextbook.Part01.Chapter03
import Mathlib.LinearAlgebra.Dual.Lemmas
import Mathlib.LinearAlgebra.Matrix.Trace

namespace CrouzeixTextbook.Part01

open scoped Matrix

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

set_option linter.defProp false in
/-- CFT-04-003: coordinate change conjugates the matrix action. -/
def coordinate_change_conjugacy := @change_basis_action

set_option linter.defProp false in
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

section DualBasisSupport

variable {𝕜 V W ι κ : Type*} [Field 𝕜] [AddCommGroup V] [Module 𝕜 V]
  [AddCommGroup W] [Module 𝕜 W]

/-- Coordinate functionals exist and form a basis of the finite algebraic dual. -/
theorem dual_basis_exists [Fintype ι] [DecidableEq ι] (b : Module.Basis ι 𝕜 V) :
    ∃ d : Module.Basis ι 𝕜 (Module.Dual 𝕜 V),
      ∀ i j, d i (b j) = if j = i then 1 else 0 :=
  ⟨b.dualBasis, b.dualBasis_apply_self⟩

theorem dual_basis_expansion [Fintype ι] (b : Module.Basis ι 𝕜 V)
    (φ : Module.Dual 𝕜 V) : φ = ∑ i, φ (b i) • b.coord i := by
  exact (b.sum_dual_apply_smul_coord φ).symm

/-- The extension itself, separate from the annihilator dimension provider. -/
noncomputable def dualityExtendedBasis {v : ι → V} (h : LinearIndependent 𝕜 v) :=
  Module.Basis.sumExtend h

theorem annihilator_of_basis_span [Fintype ι] [Fintype κ]
    (b : Module.Basis (ι ⊕ κ) 𝕜 V) (φ : Module.Dual 𝕜 V) :
    φ ∈ (Submodule.span 𝕜 (Set.range (fun i => b (Sum.inl i)))).dualAnnihilator ↔
      ∀ i, φ (b (Sum.inl i)) = 0 := by
  rw [Submodule.mem_dualAnnihilator]
  constructor
  · intro h i
    exact h _ (Submodule.subset_span ⟨i, rfl⟩)
  · intro h v hv
    have hs : Submodule.span 𝕜 (Set.range (fun i => b (Sum.inl i))) ≤ φ.ker := by
      apply Submodule.span_le.mpr
      rintro _ ⟨i, rfl⟩
      exact h i
    exact hs hv

/-- The added dual coordinates span the annihilator; this is the extension proof step. -/
theorem annihilator_expansion [Fintype ι] [Fintype κ]
    (b : Module.Basis (ι ⊕ κ) 𝕜 V) (φ : Module.Dual 𝕜 V)
    (h : ∀ i, φ (b (Sum.inl i)) = 0) :
    φ = ∑ j, φ (b (Sum.inr j)) • b.coord (Sum.inr j) := by
  classical
  calc
    φ = ∑ i, φ (b i) • b.coord i := dual_basis_expansion b φ
    _ = _ := by rw [Fintype.sum_sum_type]; simp [h]

/-- Evaluation on the added primal vectors proves independence of the added covectors. -/
theorem annihilator_coefficients_unique [Fintype ι] [Fintype κ]
    (b : Module.Basis (ι ⊕ κ) 𝕜 V) (a : κ → 𝕜)
    (h : (∑ j, a j • b.coord (Sum.inr j)) = 0) : a = 0 := by
  classical
  funext j
  have hj := congrArg (fun φ : Module.Dual 𝕜 V => φ (b (Sum.inr j))) h
  simpa [Module.Basis.coord_apply, Finsupp.single_apply] using hj

theorem annihilator_dimension [Module.Finite 𝕜 V] (U : Submodule 𝕜 V) :
    Module.finrank 𝕜 U + Module.finrank 𝕜 U.dualAnnihilator = Module.finrank 𝕜 V :=
  Subspace.finrank_add_finrank_dualAnnihilator_eq U

/-- Transpose coefficients are derived by expanding each primal basis image. -/
theorem pullback_basis_coefficients [Fintype ι] [Fintype κ] [DecidableEq ι]
    (b : Module.Basis ι 𝕜 V) (c : Module.Basis κ 𝕜 W)
    (T : V →ₗ[𝕜] W) (φ : Module.Dual 𝕜 W) (i : ι) :
    b.dualBasis.repr (T.dualMap φ) i =
      ∑ j, c.repr (T (b i)) j * φ (c j) := by
  rw [Module.Basis.dualBasis_repr, dual_map_apply]
  calc
    φ (T (b i)) = φ (∑ j, c.repr (T (b i)) j • c j) := by rw [c.sum_repr]
    _ = _ := by simp only [map_sum, map_smul, smul_eq_mul]

end DualBasisSupport

def planeFunctional (a b : ℝ) : Module.Dual ℝ (ℝ × ℝ) where
  toFun v := a * v.1 + b * v.2
  map_add' v w := by simp; ring
  map_smul' c v := by simp; ring

def triangularMap : (ℝ × ℝ) →ₗ[ℝ] (ℝ × ℝ) where
  toFun v := (v.1 + 2 * v.2, 3 * v.2)
  map_add' v w := by ext <;> simp <;> ring
  map_smul' c v := by ext <;> simp <;> ring

def secondPlaneMap : (ℝ × ℝ) →ₗ[ℝ] (ℝ × ℝ) where
  toFun v := (2 * v.1, v.1 + v.2)
  map_add' v w := by ext <;> simp <;> ring
  map_smul' c v := by ext <;> simp <;> ring

def spaceFunctional (a b c : ℝ) : Module.Dual ℝ (ℝ × ℝ × ℝ) where
  toFun v := a * v.1 + b * v.2.1 + c * v.2.2
  map_add' v w := by simp; ring
  map_smul' d v := by simp; ring

/-- The concrete Gram-diagonal pairing, with no ambient inner-product instance. -/
def weightedMetric (u v : ℝ × ℝ) : ℝ := 2 * u.1 * v.1 + u.2 * v.2

theorem weightedMetric_symmetric (u v : ℝ × ℝ) : weightedMetric u v = weightedMetric v u := by
  unfold weightedMetric
  ring

theorem weightedMetric_linear_right (u v w : ℝ × ℝ) (a : ℝ) :
    weightedMetric u (v + a • w) = weightedMetric u v + a * weightedMetric u w := by
  simp [weightedMetric]
  ring

theorem weightedMetric_positive (v : ℝ × ℝ) (hv : v ≠ 0) : 0 < weightedMetric v v := by
  have h : v.1 ≠ 0 ∨ v.2 ≠ 0 := by
    by_contra hn
    push Not at hn
    exact hv (Prod.ext hn.1 hn.2)
  rcases h with h | h
  · have := sq_pos_of_ne_zero h
    have := sq_nonneg v.2
    dsimp [weightedMetric]
    nlinarith
  · have := sq_pos_of_ne_zero h
    have := sq_nonneg v.1
    dsimp [weightedMetric]
    nlinarith

theorem redundant_predictor_pullback (a b : ℝ) (v : ℝ × ℝ × ℝ) :
    redundantPredictor.dualMap (planeFunctional a b) v =
      a * v.1 + b * v.2.1 + (a + b) * v.2.2 ∧
    redundantPredictor.dualMap (planeFunctional a b) nullDirection = 0 := by
  constructor <;> simp [redundantPredictor, planeFunctional, nullDirection]
  ring

namespace Exercises.Chapter04

theorem exercise_01_solution (v w : ℝ × ℝ) (r : ℝ) :
    planeFunctional 2 (-3) (v + r • w) =
      planeFunctional 2 (-3) v + r * planeFunctional 2 (-3) w ∧
    triangularMap.dualMap (planeFunctional 2 (-3)) v = 2 * v.1 - 5 * v.2 := by
  constructor <;> simp [planeFunctional, triangularMap] <;> ring

theorem exercise_02_solution :
    triangularMap.dualMap (planeFunctional 4 (-1)) = planeFunctional 4 5 := by
  apply LinearMap.ext
  intro v
  simp [triangularMap, planeFunctional]
  ring

theorem exercise_03_solution {𝕜 : Type*} [Field 𝕜]
    (A : Matrix (Fin 2) (Fin 2) 𝕜) (φ : Module.Dual 𝕜 (Fin 2 → 𝕜)) (i : Fin 2) :
    (Matrix.toLin' A).dualMap φ (Pi.single i 1) =
      (A.transpose *ᵥ (fun j => φ (Pi.single j 1))) i := by
  simpa [Matrix.mulVec, dotProduct, Matrix.transpose_apply, Matrix.toLin'_apply,
    Matrix.mulVec_single_one] using
    (pullback_basis_coefficients (Pi.basisFun 𝕜 (Fin 2)) (Pi.basisFun 𝕜 (Fin 2))
      (Matrix.toLin' A) φ i)

theorem exercise_04_solution (a b c : ℝ) :
    spaceFunctional a b c ∈
      (Submodule.span ℝ {((1, 1, 0) : ℝ × ℝ × ℝ)}).dualAnnihilator ↔ a + b = 0 := by
  rw [Submodule.mem_dualAnnihilator]
  constructor
  · intro h
    simpa [spaceFunctional] using h (1, 1, 0) (Submodule.subset_span (Set.mem_singleton _))
  · intro h v hv
    have hs : Submodule.span ℝ {((1, 1, 0) : ℝ × ℝ × ℝ)} ≤
        (spaceFunctional a b c).ker := by
      apply Submodule.span_le.mpr
      intro x hx
      rcases Set.mem_singleton_iff.mp hx with rfl
      simpa [spaceFunctional] using h
    exact hs hv

theorem exercise_05_solution :
    (∀ v : ℝ × ℝ, v ≠ 0 → 0 < weightedMetric v v) ∧
    (∀ x h : ℝ × ℝ, ∀ t : ℝ,
      planeFunctional 1 2 (x + t • h) - planeFunctional 1 2 x = t * (h.1 + 2 * h.2)) ∧
    (∀ h : ℝ × ℝ, weightedMetric ((1 / 2 : ℝ), 2) h = h.1 + 2 * h.2) ∧
    ((1 / 2 : ℝ), (2 : ℝ)) ≠ (1, 2) := by
  refine ⟨weightedMetric_positive, ?_, ?_, ?_⟩
  · intro x h t
    simp [planeFunctional]
    ring
  · intro h
    simp [weightedMetric]
  · norm_num

theorem exercise_06_solution (v : ℝ × ℝ) :
    (triangularMap.comp secondPlaneMap).dualMap (planeFunctional 4 (-1)) v =
      13 * v.1 + 5 * v.2 ∧
    secondPlaneMap.dualMap (triangularMap.dualMap (planeFunctional 4 (-1))) v =
      13 * v.1 + 5 * v.2 ∧
    triangularMap.dualMap (secondPlaneMap.dualMap (planeFunctional 4 (-1))) v =
      7 * v.1 + 11 * v.2 := by
  refine ⟨?_, ?_, ?_⟩ <;>
    simp [LinearMap.comp_apply, triangularMap, secondPlaneMap, planeFunctional] <;>
    ring

end Exercises.Chapter04

end CrouzeixTextbook.Part01

#check CrouzeixTextbook.Part01.Exercises.Chapter04.exercise_01_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter04.exercise_02_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter04.exercise_03_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter04.exercise_04_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter04.exercise_05_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter04.exercise_06_solution
