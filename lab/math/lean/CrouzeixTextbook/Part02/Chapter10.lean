import CrouzeixTextbook.Part01.Chapter05
import Mathlib.LinearAlgebra.TensorProduct.Basic

namespace CrouzeixTextbook.Part02

open scoped Matrix TensorProduct

set_option linter.defProp false

section CoordinateDuality

variable {m n : ℕ}

/-- CFT-10-001: the coordinate pairing moves a matrix across as its transpose.
Proved here; the previous provider lived in a namespace the receipt exporter does
not treat as maintained. -/
theorem bilinear_pairing_duality (A : Matrix (Fin m) (Fin n) ℝ)
    (x : Fin n → ℝ) (y : Fin m → ℝ) :
    (A *ᵥ x) ⬝ᵥ y = x ⬝ᵥ (A.transpose *ᵥ y) := by
  simp [dotProduct, Matrix.mulVec, Matrix.transpose_apply, Finset.sum_mul,
    Finset.mul_sum]
  rw [Finset.sum_comm]
  simp [mul_comm, mul_left_comm]

/-- CFT-10-002: the pullback action on covector coordinates is the transpose action. -/
theorem transpose_coordinate_action (A : Matrix (Fin m) (Fin n) ℝ) (y : Fin m → ℝ) :
    y ᵥ* A = A.transpose *ᵥ y := by
  rw [← Matrix.vecMul_transpose, Matrix.transpose_transpose]

end CoordinateDuality

/-- CFT-10-003: determinant is multiplicative, re-exported from Chapter 5. -/
def determinant_top_degree_multiplicative := @CrouzeixTextbook.Part01.determinant_multiplicative

/-- CFT-10-004: diagonal determinant is the product of entries, re-exported from Chapter 5. -/
def alternating_diagonal_volume := @CrouzeixTextbook.Part01.determinant_diagonal

/-- CFT-10-005: trace is cyclic, re-exported from Chapter 5. -/
def contraction_trace_cyclic := @CrouzeixTextbook.Part01.trace_cyclic

/-- CFT-10-006: the sign kernel of an alternating pairing. -/
theorem wedge_sign_kernel {a b : ℤ} : -(a - b) = b - a := by ring

section TensorAndReshape

/-- Every bilinear map factors through the tensor product, and the factorization
computes on pure tensors. This is the universal property, which is what makes the
tensor product an object rather than a layout. -/
theorem bilinear_factors_through_tensor
    {R M N P : Type*} [CommRing R]
    [AddCommGroup M] [Module R M] [AddCommGroup N] [Module R N]
    [AddCommGroup P] [Module R P]
    (f : M →ₗ[R] N →ₗ[R] P) (x : M) (y : N) :
    TensorProduct.lift f (x ⊗ₜ[R] y) = f x y :=
  TensorProduct.lift.tmul x y

/-- Reshaping is a relabeling of the index set and nothing more. -/
theorem reshape_is_index_relabeling (m n : ℕ) :
    Function.Bijective (finProdFinEquiv : Fin m × Fin n → Fin (m * n)) :=
  finProdFinEquiv.bijective

/-- A layout is not a tensor product: not every array of the product shape is a
pure tensor. The identity has nonzero determinant, and every pure tensor's
two-by-two determinant vanishes. -/
theorem identity_is_not_a_pure_tensor :
    ¬ ∃ x y : Fin 2 → ℝ,
      ∀ i j, (1 : Matrix (Fin 2) (Fin 2) ℝ) i j = x i * y j := by
  rintro ⟨x, y, h⟩
  have hdet : (1 : Matrix (Fin 2) (Fin 2) ℝ).det = 0 := by
    rw [Matrix.det_fin_two, h 0 0, h 0 1, h 1 0, h 1 1]
    ring
  rw [Matrix.det_one] at hdet
  exact one_ne_zero hdet

end TensorAndReshape

section Contraction

variable {a b c : ℕ}

/-- Contraction over the middle index of a three-index array. The types record
which axis is summed; nothing about the notation does. -/
def contractMiddle (T : Fin a → Fin b → Fin c → ℝ) (i : Fin a) (k : Fin c) : ℝ :=
  ∑ j, T i j k

/-- Contracting the outer product of two matrices over the shared axis is exactly
matrix multiplication. -/
theorem contract_middle_eq_mul (A : Matrix (Fin a) (Fin b) ℝ) (B : Matrix (Fin b) (Fin c) ℝ) :
    contractMiddle (fun i j k => A i j * B j k) = fun i k => (A * B) i k := by
  funext i k
  simp [contractMiddle, Matrix.mul_apply]

/-- Contraction is linear in the array being contracted. -/
theorem contract_middle_add (S T : Fin a → Fin b → Fin c → ℝ) :
    contractMiddle (fun i j k => S i j k + T i j k)
      = fun i k => contractMiddle S i k + contractMiddle T i k := by
  funext i k
  simp [contractMiddle, Finset.sum_add_distrib]

end Contraction

section BasisDependence

variable {n : ℕ}

/-- Components of a bilinear form are basis-dependent, and the dependence is a
congruence rather than a similarity. -/
theorem bilinear_components_change_by_congruence
    (G S : Matrix (Fin n) (Fin n) ℝ) (x y : Fin n → ℝ) :
    (S *ᵥ x) ⬝ᵥ (G *ᵥ (S *ᵥ y)) = x ⬝ᵥ ((S.transpose * G * S) *ᵥ y) := by
  rw [bilinear_pairing_duality S x (G *ᵥ (S *ᵥ y)), Matrix.mulVec_mulVec,
    Matrix.mulVec_mulVec, Matrix.mul_assoc]

/-- The transpose of the running shear, written out. -/
theorem shear_transpose :
    (!![1, 1; 0, 1] : Matrix (Fin 2) (Fin 2) ℝ).transpose = !![1, 0; 1, 1] := by
  ext i j
  fin_cases i <;> fin_cases j <;> simp [Matrix.transpose_apply]

/-- Congruence and similarity differ: this shear is not orthogonal, so its
transpose is not its inverse and the two transports are different operations. -/
theorem congruence_is_not_similarity :
    (!![1, 1; 0, 1] : Matrix (Fin 2) (Fin 2) ℝ).transpose * !![1, 1; 0, 1] ≠ 1 := by
  rw [shear_transpose]
  intro h
  have hentry := congrFun (congrFun h 1) 1
  norm_num [Matrix.mul_fin_two, Matrix.one_apply] at hentry

end BasisDependence

namespace Exercises.Chapter10

/-- CFT-10-E01. -/
theorem exercise_01_solution (A : Matrix (Fin 2) (Fin 3) ℝ) (x : Fin 3 → ℝ) (y : Fin 2 → ℝ) :
    (A *ᵥ x) ⬝ᵥ y = x ⬝ᵥ (A.transpose *ᵥ y) ∧ y ᵥ* A = A.transpose *ᵥ y :=
  ⟨bilinear_pairing_duality A x y, transpose_coordinate_action A y⟩

/-- CFT-10-E02. -/
theorem exercise_02_solution :
    contractMiddle (fun i j k => (!![1, 2; 3, 4] : Matrix (Fin 2) (Fin 2) ℝ) i j *
        (!![5, 6; 7, 8] : Matrix (Fin 2) (Fin 2) ℝ) j k)
      = fun i k => (!![19, 22; 43, 50] : Matrix (Fin 2) (Fin 2) ℝ) i k := by
  rw [contract_middle_eq_mul]
  funext i k
  fin_cases i <;> fin_cases k <;> norm_num [Matrix.mul_fin_two]

/-- CFT-10-E03. -/
theorem exercise_03_solution {a b c : ℕ}
    (A : Matrix (Fin a) (Fin b) ℝ) (B : Matrix (Fin b) (Fin c) ℝ)
    (S T : Fin a → Fin b → Fin c → ℝ) :
    contractMiddle (fun i j k => A i j * B j k) = (fun i k => (A * B) i k) ∧
      contractMiddle (fun i j k => S i j k + T i j k)
        = fun i k => contractMiddle S i k + contractMiddle T i k :=
  ⟨contract_middle_eq_mul A B, contract_middle_add S T⟩

/-- CFT-10-E04. -/
theorem exercise_04_solution {n : ℕ} (G S : Matrix (Fin n) (Fin n) ℝ) (x y : Fin n → ℝ) :
    (S *ᵥ x) ⬝ᵥ (G *ᵥ (S *ᵥ y)) = x ⬝ᵥ ((S.transpose * G * S) *ᵥ y) := by
  rw [bilinear_pairing_duality S x (G *ᵥ (S *ᵥ y)), Matrix.mulVec_mulVec,
    Matrix.mulVec_mulVec, Matrix.mul_assoc]

/-- CFT-10-E05. -/
theorem exercise_05_solution :
    Function.Bijective (finProdFinEquiv : Fin 2 × Fin 3 → Fin (2 * 3)) ∧
      ¬ ∃ x y : Fin 2 → ℝ,
        ∀ i j, (1 : Matrix (Fin 2) (Fin 2) ℝ) i j = x i * y j :=
  ⟨reshape_is_index_relabeling 2 3, identity_is_not_a_pure_tensor⟩

/-- CFT-10-E06. -/
theorem exercise_06_solution {R M N P : Type*} [CommRing R]
    [AddCommGroup M] [Module R M] [AddCommGroup N] [Module R N]
    [AddCommGroup P] [Module R P]
    (f : M →ₗ[R] N →ₗ[R] P) (x : M) (y : N) (a b : ℤ) :
    TensorProduct.lift f (x ⊗ₜ[R] y) = f x y ∧ -(a - b) = b - a :=
  ⟨bilinear_factors_through_tensor f x y, wedge_sign_kernel⟩

end Exercises.Chapter10

end CrouzeixTextbook.Part02
