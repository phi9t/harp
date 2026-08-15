import Mathlib.Algebra.Module.Submodule.Ker
import Mathlib.Data.Real.Basic

namespace MathematicalFoundations.Linear

theorem linear_map_zero {V W : Type*} [AddCommGroup V] [AddCommGroup W]
    [Module ℝ V] [Module ℝ W] (f : V →ₗ[ℝ] W) : f 0 = 0 :=
  f.map_zero

theorem linear_map_add {V W : Type*} [AddCommGroup V] [AddCommGroup W]
    [Module ℝ V] [Module ℝ W] (f : V →ₗ[ℝ] W) (x y : V) :
    f (x + y) = f x + f y :=
  f.map_add x y

theorem linear_map_smul {V W : Type*} [AddCommGroup V] [AddCommGroup W]
    [Module ℝ V] [Module ℝ W] (f : V →ₗ[ℝ] W) (a : ℝ) (x : V) :
    f (a • x) = a • f x :=
  f.map_smul a x

theorem linear_kernel_zero {V W : Type*} [AddCommGroup V] [AddCommGroup W]
    [Module ℝ V] [Module ℝ W] (f : V →ₗ[ℝ] W) : (0 : V) ∈ f.ker :=
  f.ker.zero_mem

theorem linear_kernel_add {V W : Type*} [AddCommGroup V] [AddCommGroup W]
    [Module ℝ V] [Module ℝ W] (f : V →ₗ[ℝ] W) {x y : V}
    (hx : x ∈ f.ker) (hy : y ∈ f.ker) : x + y ∈ f.ker :=
  f.ker.add_mem hx hy

theorem linear_kernel_smul {V W : Type*} [AddCommGroup V] [AddCommGroup W]
    [Module ℝ V] [Module ℝ W] (f : V →ₗ[ℝ] W) (a : ℝ) {x : V}
    (hx : x ∈ f.ker) : a • x ∈ f.ker :=
  f.ker.smul_mem a hx

theorem linear_finite_coordinate_reconstruction {n : ℕ} {x y : Fin n → ℝ}
    (coordinates : ∀ i : Fin n, x i = y i) : x = y := by
  funext i
  exact coordinates i

end MathematicalFoundations.Linear
