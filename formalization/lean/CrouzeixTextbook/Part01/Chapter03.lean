import CrouzeixTextbook.Part01.Chapter02
import Mathlib.LinearAlgebra.FiniteDimensional.Lemmas
import Mathlib.LinearAlgebra.Basis.VectorSpace
import Mathlib.LinearAlgebra.Isomorphisms

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
  constructor
  · intro h
    apply le_antisymm
    · intro x hx
      exact h (show T x = T 0 by simpa using hx)
    · exact bot_le
  · intro h x y hxy
    have hz : x - y ∈ T.ker := by
      rw [LinearMap.mem_ker, map_sub, hxy, sub_self]
    rw [h] at hz
    exact sub_eq_zero.mp hz

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

section StructuralSupport

variable {𝕜 V W : Type*} [Field 𝕜] [AddCommGroup V] [Module 𝕜 V]
  [AddCommGroup W] [Module 𝕜 W]

theorem kernel_range_closure (T : V →ₗ[𝕜] W) :
    (0 : V) ∈ T.ker ∧
    (∀ x y : V, x ∈ T.ker → y ∈ T.ker → x + y ∈ T.ker) ∧
    (∀ a : 𝕜, ∀ x : V, x ∈ T.ker → a • x ∈ T.ker) ∧
    (0 : W) ∈ T.range ∧
    (∀ x y : W, x ∈ T.range → y ∈ T.range → x + y ∈ T.range) ∧
    (∀ a : 𝕜, ∀ x : W, x ∈ T.range → a • x ∈ T.range) := by
  exact ⟨T.ker.zero_mem, fun _ _ => T.ker.add_mem,
    fun a _ => T.ker.smul_mem a, T.range.zero_mem,
    fun _ _ => T.range.add_mem, fun a _ => T.range.smul_mem a⟩

/-- Applying T to the decomposition supplied by an extended kernel basis gives spanning. -/
theorem image_spanning_from_decomposition {ι : Type*} [Fintype ι]
    (T : V →ₗ[𝕜] W) (v : ι → V)
    (hdecomp : ∀ x : V, ∃ k ∈ T.ker, ∃ a : ι → 𝕜,
      x = k + ∑ i, a i • v i) (y : T.range) :
    ∃ a : ι → 𝕜, (y : W) = ∑ i, a i • T (v i) := by
  obtain ⟨x, hx⟩ := y.property
  obtain ⟨k, hk, a, ha⟩ := hdecomp x
  refine ⟨a, ?_⟩
  rw [← hx, ha, map_add, map_sum]
  simp only [LinearMap.mem_ker.mp hk, zero_add, map_smul]

/-- The independence consequence of an extended kernel basis survives applying T. -/
theorem image_independence_from_kernel {ι : Type*} [Fintype ι]
    (T : V →ₗ[𝕜] W) (v : ι → V)
    (hind : ∀ a : ι → 𝕜, (∑ i, a i • v i) ∈ T.ker → a = 0)
    (a : ι → 𝕜) (ha : (∑ i, a i • T (v i)) = 0) : a = 0 := by
  apply hind a
  rw [LinearMap.mem_ker, map_sum]
  simpa only [map_smul] using ha

end StructuralSupport

section FirstIsomorphism

variable {𝕜 V W : Type*} [Ring 𝕜] [AddCommGroup V] [Module 𝕜 V]
  [AddCommGroup W] [Module 𝕜 W]

/-- Constructed by Mathlib's quotient lift, without choosing a complement. -/
noncomputable def firstIso (T : V →ₗ[𝕜] W) : (V ⧸ T.ker) ≃ₗ[𝕜] T.range :=
  T.quotKerEquivRange

theorem firstIso_apply (T : V →ₗ[𝕜] W) (x : V) :
    (firstIso T (T.ker.mkQ x) : W) = T x := by
  rfl

theorem firstIso_inverse_class (T : V →ₗ[𝕜] W) (x : V)
    (hx : T x ∈ T.range) : (firstIso T).symm ⟨T x, hx⟩ = T.ker.mkQ x := by
  exact T.quotKerEquivRange_symm_apply_image x hx

theorem firstIso_bijective (T : V →ₗ[𝕜] W) : Function.Bijective (firstIso T) := by
  exact (firstIso T).bijective

theorem firstIso_well_defined (T : V →ₗ[𝕜] W) (x y : V)
    (h : x - y ∈ T.ker) : T x = T y := by
  apply sub_eq_zero.mp
  rw [← map_sub]
  exact h

theorem firstIso_linear_operations (T : V →ₗ[𝕜] W) (a : 𝕜)
    (q r : V ⧸ T.ker) :
    firstIso T (q + r) = firstIso T q + firstIso T r ∧
    firstIso T (a • q) = a • firstIso T q := by
  exact ⟨map_add _ q r, map_smul _ a q⟩

end FirstIsomorphism

theorem invariantRestriction_apply {𝕜 V : Type*} [Semiring 𝕜]
    [AddCommMonoid V] [Module 𝕜 V] (T : V →ₗ[𝕜] V) (W : Submodule 𝕜 V)
    (hW : ∀ x ∈ W, T x ∈ W) (x : W) :
    (invariantRestriction T W hW x : V) = T x := by
  rfl

def firstCoordinateLine : Submodule ℝ (ℝ × ℝ) where
  carrier := {v | v.2 = 0}
  zero_mem' := rfl
  add_mem' hx hy := by dsimp at *; rw [hx, hy]; simp
  smul_mem' a v hv := by dsimp at *; rw [hv]; simp

theorem nilpotent_line_invariant (α : ℝ) :
    ∀ v ∈ firstCoordinateLine, triangularAction 0 α v ∈ firstCoordinateLine := by
  intro v _
  change 0 * v.2 = 0
  simp

theorem nilpotent_kernel_range (α : ℝ) (hα : α ≠ 0) :
    (triangularAction 0 α).ker = firstCoordinateLine ∧
    (triangularAction 0 α).range = firstCoordinateLine := by
  constructor
  · ext v
    change triangularAction 0 α v = 0 ↔ v.2 = 0
    simp [triangularAction, Prod.ext_iff, hα]
  · ext v
    change (∃ x, triangularAction 0 α x = v) ↔ v.2 = 0
    constructor
    · rintro ⟨x, rfl⟩
      simp [triangularAction]
    · intro hv
      refine ⟨(0, v.1 / α), ?_⟩
      apply Prod.ext
      · change 0 * 0 + α * (v.1 / α) = v.1
        field_simp
        simp
      · simpa [triangularAction] using hv.symm

theorem nilpotent_square (α : ℝ) : (triangularAction 0 α) ^ 2 = 0 := by
  apply LinearMap.ext
  intro v
  simp [pow_two, Module.End.mul_apply, triangularAction]

theorem nilpotent_zero_case :
    (triangularAction 0 0).ker = ⊤ ∧ (triangularAction 0 0).range = ⊥ := by
  have hz : triangularAction 0 0 = 0 := by
    apply LinearMap.ext
    intro v
    simp [triangularAction]
  rw [hz]
  simp

theorem nilpotent_powers (α : ℝ) (n : ℕ) : (triangularAction 0 α) ^ (n + 2) = 0 := by
  rw [pow_add, nilpotent_square, mul_zero]

/-- The identity power has trivial kernel and full range, for every parameter. -/
theorem nilpotent_power_zero_kernel_range (α : ℝ) :
    ((triangularAction 0 α) ^ 0).ker = ⊥ ∧
    ((triangularAction 0 α) ^ 0).range = ⊤ := by
  change (LinearMap.id : (ℝ × ℝ) →ₗ[ℝ] (ℝ × ℝ)).ker = ⊥ ∧
    (LinearMap.id : (ℝ × ℝ) →ₗ[ℝ] (ℝ × ℝ)).range = ⊤
  simp

/-- Every power at least two has full kernel and zero range, including alpha zero. -/
theorem nilpotent_higher_power_kernel_range (α : ℝ) (n : ℕ) :
    ((triangularAction 0 α) ^ (n + 2)).ker = ⊤ ∧
    ((triangularAction 0 α) ^ (n + 2)).range = ⊥ := by
  rw [nilpotent_powers]
  simp

theorem nilpotent_restriction (α : ℝ) :
    invariantRestriction (triangularAction 0 α) firstCoordinateLine
      (nilpotent_line_invariant α) = 0 := by
  apply LinearMap.ext
  intro v
  apply Subtype.ext
  have hv : (v : ℝ × ℝ).2 = 0 := v.property
  change triangularAction 0 α (v : ℝ × ℝ) = 0
  simp [triangularAction, hv]

namespace Exercises.Chapter03

/-- E01: the exact zero fiber of the running map. -/
theorem exercise_01_solution (v : ℝ × ℝ × ℝ) :
    redundantPredictor v = 0 ↔ v = (-v.2.2, -v.2.2, v.2.2) := by
  constructor
  · intro h
    have hx := congrArg Prod.fst h
    have hy := congrArg Prod.snd h
    simp [redundantPredictor] at hx hy
    ext <;> dsimp <;> linarith
  · intro h
    have hx := congrArg Prod.fst h
    have hy := congrArg (fun w : ℝ × ℝ × ℝ => w.2.1) h
    simp only at hx hy
    simp [redundantPredictor, hx, hy]

/-- E02: explicit spanning and independence certificates for both bases. -/
theorem exercise_02_solution :
    redundantPredictor.ker = Submodule.span ℝ {nullDirection} ∧
    redundantPredictor.range = ⊤ ∧
    (∀ a : ℝ, a • nullDirection = 0 → a = 0) ∧
    (∀ p q : ℝ, p • ((1, 0) : ℝ × ℝ) + q • ((0, 1) : ℝ × ℝ) = (p, q)) ∧
    (∀ a b : ℝ, a • ((1, 0) : ℝ × ℝ) + b • ((0, 1) : ℝ × ℝ) = 0 →
      a = 0 ∧ b = 0) := by
  refine ⟨redundantPredictor_kernel, ?_, ?_, ?_, ?_⟩
  · apply top_unique
    intro v _
    exact ⟨(v.1, v.2, 0), by simp [redundantPredictor]⟩
  · intro a h
    have hz := congrArg (fun w : ℝ × ℝ × ℝ => w.2.2) h
    simpa [nullDirection] using hz
  · intro p q
    simp
  · intro a b h
    simpa [Prod.ext_iff] using h

/-- E03: inverse image classes and the explicit z=0 representative. -/
theorem exercise_03_solution (v : ℝ × ℝ × ℝ) :
    (firstIso redundantPredictor).symm ⟨redundantPredictor v, ⟨v, rfl⟩⟩ =
      redundantPredictor.ker.mkQ v ∧
    (firstIso redundantPredictor).symm ⟨redundantPredictor v, ⟨v, rfl⟩⟩ =
      redundantPredictor.ker.mkQ (v.1 + v.2.2, v.2.1 + v.2.2, 0) := by
  have hi := firstIso_inverse_class redundantPredictor v ⟨v, rfl⟩
  refine ⟨hi, hi.trans ?_⟩
  apply (quotient_class_eq_iff redundantPredictor.ker _ _).mpr
  rw [LinearMap.mem_ker]
  simp [redundantPredictor]

/-- E04: the running map is injective when both inputs have z=0. -/
theorem exercise_04_solution (v w : ℝ × ℝ × ℝ)
    (hv : v.2.2 = 0) (hw : w.2.2 = 0)
    (h : redundantPredictor v = redundantPredictor w) : v = w := by
  have hx := congrArg Prod.fst h
  have hy := congrArg Prod.snd h
  simp [redundantPredictor, hv, hw] at hx hy
  ext
  · exact hx
  · exact hy
  · rw [hv, hw]

/-- E05: range inclusion can be strict even for a final identity map. -/
theorem exercise_05_solution :
    LinearMap.range ((LinearMap.id : ℝ →ₗ[ℝ] ℝ).comp (0 : ℝ →ₗ[ℝ] ℝ)) <
      LinearMap.range (LinearMap.id : ℝ →ₗ[ℝ] ℝ) := by
  simp

/-- E06: restriction commutes with every nonnegative power, proved by induction. -/
theorem exercise_06_solution {𝕜 V : Type*} [Semiring 𝕜]
    [AddCommMonoid V] [Module 𝕜 V] (T : V →ₗ[𝕜] V) (W : Submodule 𝕜 V)
    (hW : ∀ x ∈ W, T x ∈ W) (n : ℕ) (x : W) :
    (((invariantRestriction T W hW) ^ n) x : V) = (T ^ n) (x : V) := by
  induction n with
  | zero => rfl
  | succ n ih =>
      simp only [pow_succ', Module.End.mul_apply, invariantRestriction_apply, ih]

end Exercises.Chapter03

end CrouzeixTextbook.Part01

#check CrouzeixTextbook.Part01.Exercises.Chapter03.exercise_01_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter03.exercise_02_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter03.exercise_03_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter03.exercise_04_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter03.exercise_05_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter03.exercise_06_solution
