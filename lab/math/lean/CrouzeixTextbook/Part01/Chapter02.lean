import Mathlib.LinearAlgebra.Dimension.Finite
import Mathlib.LinearAlgebra.Basis.Defs
import Mathlib.LinearAlgebra.Quotient.Basic
import Mathlib.Tactic

namespace CrouzeixTextbook.Part01

/-- CFT-02-001: the span is the least subspace containing the generators. -/
theorem span_minimality {𝕜 V : Type*} [Semiring 𝕜] [AddCommMonoid V]
    [Module 𝕜 V] {s : Set V} {W : Submodule 𝕜 V} (h : s ⊆ W) :
    Submodule.span 𝕜 s ≤ W :=
  Submodule.span_le.mpr h

/-- CFT-02-002: intersection membership means satisfying both constraints. -/
theorem mem_subspace_intersection_iff {𝕜 V : Type*} [Semiring 𝕜]
    [AddCommMonoid V] [Module 𝕜 V] (U W : Submodule 𝕜 V) (x : V) :
    x ∈ U ⊓ W ↔ x ∈ U ∧ x ∈ W := by
  constructor
  · intro hx
    exact ⟨hx.1, hx.2⟩
  · rintro ⟨hu, hw⟩
    exact ⟨hu, hw⟩

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
    Function.Surjective W.mkQ := by
  intro q
  obtain ⟨x, hx⟩ := Submodule.mkQ_surjective W q
  exact ⟨x, hx⟩

section StructuralSupport

variable {𝕜 V : Type*} [Field 𝕜] [AddCommGroup V] [Module 𝕜 V]

/-- The finite-combination construction and intersection definition agree. -/
theorem span_finite_combination (s : Set V) (x : V) :
    x ∈ Submodule.span 𝕜 s ↔
      ∃ c : s →₀ 𝕜, Finsupp.linearCombination 𝕜 (fun v : s => (v : V)) c = x := by
  exact Finsupp.mem_span_iff_linearCombination 𝕜 s x

theorem span_as_intersection (s : Set V) :
    Submodule.span 𝕜 s = sInf {W : Submodule 𝕜 V | s ⊆ W} := by
  rfl

/-- Addition and scaling of finite coefficients construct the corresponding vectors. -/
theorem finite_combination_closure (s : Set V) (c d : s →₀ 𝕜) (a : 𝕜) :
    Finsupp.linearCombination 𝕜 (fun v : s => (v : V)) (c + d) =
      Finsupp.linearCombination 𝕜 (fun v : s => (v : V)) c +
        Finsupp.linearCombination 𝕜 (fun v : s => (v : V)) d ∧
    Finsupp.linearCombination 𝕜 (fun v : s => (v : V)) (a • c) =
      a • Finsupp.linearCombination 𝕜 (fun v : s => (v : V)) c := by
  exact ⟨map_add _ c d, map_smul _ a c⟩

theorem intersection_closure (U W : Submodule 𝕜 V) :
    (0 : V) ∈ U ⊓ W ∧
      (∀ x y : V, x ∈ U ⊓ W → y ∈ U ⊓ W → x + y ∈ U ⊓ W) ∧
      (∀ a : 𝕜, ∀ x : V, x ∈ U ⊓ W → a • x ∈ U ⊓ W) := by
  exact ⟨⟨U.zero_mem, W.zero_mem⟩,
    fun _ _ hx hy => ⟨U.add_mem hx.1 hy.1, W.add_mem hx.2 hy.2⟩,
    fun a _ hx => ⟨U.smul_mem a hx.1, W.smul_mem a hx.2⟩⟩

theorem basis_expansion_exists_unique {ι : Type*} (b : Module.Basis ι 𝕜 V) (x : V) :
    ∃! c : ι →₀ 𝕜, Finsupp.linearCombination 𝕜 b c = x := by
  refine ⟨b.repr x, b.linearCombination_repr x, ?_⟩
  intro c hc
  calc
    c = b.repr (Finsupp.linearCombination 𝕜 b c) := (b.repr_linearCombination c).symm
    _ = b.repr x := congrArg b.repr hc

theorem basis_transport {ι W : Type*} [AddCommGroup W] [Module 𝕜 W]
    (b : Module.Basis ι 𝕜 V) (e : V ≃ₗ[𝕜] W) (i : ι) :
    b.map e i = e (b i) := by
  exact b.map_apply e i

theorem direct_sum_existence (U W : Submodule 𝕜 V) (h : U ⊔ W = ⊤) (x : V) :
    ∃ u ∈ U, ∃ w ∈ W, u + w = x := by
  apply Submodule.mem_sup.mp
  rw [h]
  trivial

theorem quotient_relation_laws (W : Submodule 𝕜 V) :
    (∀ x : V, x - x ∈ W) ∧
      (∀ x y : V, x - y ∈ W → y - x ∈ W) ∧
      (∀ x y z : V, x - y ∈ W → y - z ∈ W → x - z ∈ W) := by
  refine ⟨fun x => by simp, ?_, ?_⟩
  · intro x y h
    simpa only [neg_sub] using W.neg_mem h
  · intro x y z hxy hyz
    convert W.add_mem hxy hyz using 1; abel

theorem quotient_operations_well_defined (W : Submodule 𝕜 V)
    (x x' y y' : V) (a : 𝕜) (hx : x - x' ∈ W) (hy : y - y' ∈ W) :
    (x + y) - (x' + y') ∈ W ∧ a • x - a • x' ∈ W := by
  constructor
  · convert W.add_mem hx hy using 1; abel
  · simpa only [smul_sub] using W.smul_mem a hx

theorem quotient_projection_kernel (W : Submodule 𝕜 V) :
    LinearMap.ker W.mkQ = W := by
  exact Submodule.ker_mkQ W

theorem quotient_class_eq_iff (W : Submodule 𝕜 V) (x y : V) :
    W.mkQ x = W.mkQ y ↔ x - y ∈ W := by
  exact Submodule.Quotient.eq W

theorem quotient_factor_well_defined {Z : Type*} [AddCommGroup Z] [Module 𝕜 Z]
    (W : Submodule 𝕜 V) (f : V →ₗ[𝕜] Z) (h : W ≤ LinearMap.ker f)
    {x y : V} (hxy : x - y ∈ W) : f x = f y := by
  apply sub_eq_zero.mp
  rw [← f.map_sub]
  exact h hxy

/-- Existence includes linearity in the type; uniqueness uses representatives. -/
theorem quotient_factor_exists_unique {Z : Type*} [AddCommGroup Z] [Module 𝕜 Z]
    (W : Submodule 𝕜 V) (f : V →ₗ[𝕜] Z) (h : W ≤ LinearMap.ker f) :
    ∃! g : (V ⧸ W) →ₗ[𝕜] Z, g.comp W.mkQ = f := by
  refine ⟨W.liftQ f h, W.liftQ_mkQ f h, ?_⟩
  intro g hg
  apply LinearMap.ext
  intro q
  obtain ⟨x, rfl⟩ := quotient_projection_surjective W q
  exact DFunLike.congr_fun hg x

end StructuralSupport

/-- The running predictor sends three real parameters to two coefficients. -/
def redundantPredictor : (ℝ × ℝ × ℝ) →ₗ[ℝ] (ℝ × ℝ) where
  toFun v := (v.1 + v.2.2, v.2.1 + v.2.2)
  map_add' v w := by ext <;> simp <;> ring
  map_smul' a v := by ext <;> simp <;> ring

def nullDirection : ℝ × ℝ × ℝ := (-1, -1, 1)

theorem redundantPredictor_kernel :
    LinearMap.ker redundantPredictor = Submodule.span ℝ {nullDirection} := by
  ext v
  rw [LinearMap.mem_ker, Submodule.mem_span_singleton]
  constructor
  · intro h
    have hx := congrArg Prod.fst h
    have hy := congrArg Prod.snd h
    simp [redundantPredictor] at hx hy
    refine ⟨v.2.2, ?_⟩
    ext <;> simp [nullDirection] <;> linarith
  · rintro ⟨a, rfl⟩
    simp [redundantPredictor, nullDirection]

/-- Coordinate action of the real matrix [[lam, alpha], [0, lam]]. -/
def triangularAction (lam α : ℝ) : (ℝ × ℝ) →ₗ[ℝ] (ℝ × ℝ) where
  toFun v := (lam * v.1 + α * v.2, lam * v.2)
  map_add' v w := by ext <;> simp <;> ring
  map_smul' a v := by ext <;> simp <;> ring

theorem triangularAction_structure (lam α : ℝ) (v : ℝ × ℝ) :
    triangularAction lam α (v.1, 0) = (lam * v.1, 0) ∧
      (triangularAction lam α v).2 = lam * v.2 ∧
      (triangularAction lam α v - lam • v) = (α * v.2, 0) ∧
      triangularAction lam α (triangularAction lam α v - lam • v) -
        lam • (triangularAction lam α v - lam • v) = 0 := by
  simp [triangularAction, Prod.ext_iff]

theorem triangularAction_no_invariant_complement (lam α : ℝ) (hα : α ≠ 0)
    (t c : ℝ) : triangularAction lam α (t, 1) ≠ c • (t, 1) := by
  intro h
  have hy := congrArg Prod.snd h
  have hx := congrArg Prod.fst h
  simp [triangularAction] at hx hy
  apply hα
  rw [← hy] at hx
  linarith

theorem triangularAction_eigenline (lam α : ℝ) (hα : α ≠ 0) (v : ℝ × ℝ) :
    triangularAction lam α v = lam • v ↔ v.2 = 0 := by
  constructor
  · intro h
    have hx := congrArg Prod.fst h
    simp [triangularAction] at hx
    exact hx.resolve_left hα
  · intro h
    ext <;> simp [triangularAction, h]

def diagonalSubspace : Submodule ℝ (ℝ × ℝ) where
  carrier := {v | v.1 = v.2}
  zero_mem' := rfl
  add_mem' hx hy := by dsimp at *; rw [hx, hy]
  smul_mem' a v hv := by dsimp at *; rw [hv]

def differenceFunctional : (ℝ × ℝ) →ₗ[ℝ] ℝ where
  toFun v := v.1 - v.2
  map_add' v w := by simp; ring
  map_smul' a v := by simp; ring

namespace Exercises.Chapter02

/-- E01: check all three subspace conditions for the concrete diagonal. -/
theorem exercise_01_solution :
    ((0 : ℝ × ℝ).1 = (0 : ℝ × ℝ).2) ∧
      (∀ u v : ℝ × ℝ, u.1 = u.2 → v.1 = v.2 → (u + v).1 = (u + v).2) ∧
      (∀ a : ℝ, ∀ u : ℝ × ℝ, u.1 = u.2 → (a • u).1 = (a • u).2) := by
  refine ⟨rfl, ?_, ?_⟩
  · intro u v hu hv
    change u.1 + v.1 = u.2 + v.2
    rw [hu, hv]
  · intro a u hu
    change a * u.1 = a * u.2
    rw [hu]

/-- E02: explicit coefficients, and consequently the whole plane is spanned. -/
theorem exercise_02_solution (x y : ℝ) :
    ((x + y) / 2) • ((1, 1) : ℝ × ℝ) + ((x - y) / 2) • ((1, -1) : ℝ × ℝ) = (x, y) ∧
      Submodule.span ℝ {((1, 1) : ℝ × ℝ), ((1, -1) : ℝ × ℝ)} = ⊤ := by
  have coords (a b : ℝ) :
      ((a + b) / 2) • ((1, 1) : ℝ × ℝ) +
        ((a - b) / 2) • ((1, -1) : ℝ × ℝ) = (a, b) := by
    ext <;> simp <;> ring
  refine ⟨coords x y, ?_⟩
  apply top_unique
  intro v _
  change (v.1, v.2) ∈ _
  rw [← coords v.1 v.2]
  exact Submodule.add_mem _
    (Submodule.smul_mem _ _ (Submodule.subset_span (by simp)))
    (Submodule.smul_mem _ _ (Submodule.subset_span (by simp)))

/-- E03: witness each new generator as a combination of the old generators. -/
theorem exercise_03_solution :
    Submodule.span ℝ {((2, 0) : ℝ × ℝ), ((0, 2) : ℝ × ℝ)} ≤
      Submodule.span ℝ {((1, 1) : ℝ × ℝ), ((1, -1) : ℝ × ℝ)} := by
  apply Submodule.span_le.mpr
  intro v hv
  have hp : ((1, 1) : ℝ × ℝ) ∈
      Submodule.span ℝ {((1, 1) : ℝ × ℝ), ((1, -1) : ℝ × ℝ)} :=
    Submodule.subset_span (by simp)
  have hm : ((1, -1) : ℝ × ℝ) ∈
      Submodule.span ℝ {((1, 1) : ℝ × ℝ), ((1, -1) : ℝ × ℝ)} :=
    Submodule.subset_span (by simp)
  simp only [Set.mem_insert_iff, Set.mem_singleton_iff] at hv
  rcases hv with rfl | rfl
  · convert Submodule.add_mem _ hp hm using 1; norm_num
  · convert Submodule.sub_mem _ hp hm using 1; norm_num

/-- E04: unique linear factor of x-y through the diagonal quotient. -/
theorem exercise_04_solution :
    ∃! g : ((ℝ × ℝ) ⧸ diagonalSubspace) →ₗ[ℝ] ℝ,
      ∀ v : ℝ × ℝ, g (diagonalSubspace.mkQ v) = v.1 - v.2 := by
  have h : diagonalSubspace ≤ LinearMap.ker differenceFunctional := by
    intro v hv
    change v.1 - v.2 = 0
    exact sub_eq_zero.mpr hv
  refine ⟨diagonalSubspace.liftQ differenceFunctional h, fun v => rfl, ?_⟩
  intro g hg
  apply LinearMap.ext
  intro q
  obtain ⟨v, rfl⟩ := quotient_projection_surjective diagonalSubspace q
  exact hg v

/-- E05: explicit witnesses in the two axes, with their sum outside the union. -/
theorem exercise_05_solution :
    ∃ u v : ℝ × ℝ,
      (u.1 = 0 ∨ u.2 = 0) ∧ (v.1 = 0 ∨ v.2 = 0) ∧
        ¬ ((u + v).1 = 0 ∨ (u + v).2 = 0) := by
  refine ⟨(1, 0), (0, 1), ?_⟩
  norm_num

/-- E06: equal outputs are exactly differences along the single null direction. -/
theorem exercise_06_solution (v w : ℝ × ℝ × ℝ) :
    redundantPredictor v = redundantPredictor w ↔
      ∃ a : ℝ, v - w = a • nullDirection := by
  constructor
  · intro h
    have hx := congrArg Prod.fst h
    have hy := congrArg Prod.snd h
    simp [redundantPredictor] at hx hy
    refine ⟨v.2.2 - w.2.2, ?_⟩
    ext <;> simp [nullDirection] <;> linarith
  · rintro ⟨a, h⟩
    have hz : redundantPredictor (v - w) = 0 := by
      rw [h]
      simp [redundantPredictor, nullDirection]
    rw [map_sub] at hz
    exact sub_eq_zero.mp hz

end Exercises.Chapter02

end CrouzeixTextbook.Part01

-- Compile clients: these names must denote the six distinct exercise solutions.
#check CrouzeixTextbook.Part01.Exercises.Chapter02.exercise_01_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter02.exercise_02_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter02.exercise_03_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter02.exercise_04_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter02.exercise_05_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter02.exercise_06_solution
