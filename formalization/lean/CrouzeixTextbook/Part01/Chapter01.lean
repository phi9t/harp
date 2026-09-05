import Mathlib.LinearAlgebra.Matrix.Charpoly.Basic
import Mathlib.Tactic

open BigOperators Matrix
open scoped Matrix

namespace CrouzeixTextbook.Part01

/-- CFT-01-001: a linear transformation is data and laws, with no chosen basis. -/
abbrev LinearTransformation
    (𝕜 V W : Type*) [Semiring 𝕜]
    [AddCommMonoid V] [AddCommMonoid W] [Module 𝕜 V] [Module 𝕜 W] :=
  V →ₗ[𝕜] W

/-- A coordinate-free structural consequence used in the first retrieval exercise. -/
theorem linear_map_preserves_zero
    {𝕜 V : Type*} [Semiring 𝕜] [AddCommMonoid V] [Module 𝕜 V]
    (T : V →ₗ[𝕜] V) : T 0 = 0 := by
  exact T.map_zero

/-- CFT-01-002: column `j` is the image of the `j`th coordinate vector. -/
theorem matrix_column_is_basis_image
    {𝕜 n : Type*} [Semiring 𝕜] [Fintype n] [DecidableEq n]
    (A : Matrix n n 𝕜) (i j : n) :
    (A *ᵥ Pi.single j 1) i = A i j := by
  exact congrFun (Matrix.mulVec_single_one A j) i

/-- The coordinate action represented by a matrix. -/
def coordinateAction
    {𝕜 n : Type*} [Semiring 𝕜] [Fintype n]
    (A : Matrix n n 𝕜) (x : n → 𝕜) : n → 𝕜 :=
  A *ᵥ x

/-- CFT-01-003: coordinate action is matrix-vector multiplication. -/
theorem coordinate_action_eq_mulVec
    {𝕜 n : Type*} [Semiring 𝕜] [Fintype n]
    (A : Matrix n n 𝕜) (x : n → 𝕜) :
    coordinateAction A x = A *ᵥ x := by
  rfl

/-- CFT-01-004: an invertible coordinate change conjugates the representing matrix. -/
theorem change_basis_action
    {𝕜 n : Type*} [CommRing 𝕜] [Fintype n] [DecidableEq n]
    (S Sinv A : Matrix n n 𝕜) (hSinv : Sinv * S = 1) (x : n → 𝕜) :
    (S * A * Sinv) *ᵥ (S *ᵥ x) = S *ᵥ (A *ᵥ x) := by
  calc
    (S * A * Sinv) *ᵥ (S *ᵥ x) = ((S * A * Sinv) * S) *ᵥ x := by
      rw [Matrix.mulVec_mulVec]
    _ = (S * A) *ᵥ x := by rw [Matrix.mul_assoc, hSinv, Matrix.mul_one]
    _ = S *ᵥ (A *ᵥ x) := by rw [Matrix.mulVec_mulVec]

/-- CFT-01-005: similarity by a matrix unit preserves the characteristic polynomial. -/
theorem similarity_preserves_charpoly
    {𝕜 n : Type*} [CommRing 𝕜] [Fintype n] [DecidableEq n]
    (S : (Matrix n n 𝕜)ˣ) (A : Matrix n n 𝕜) :
    (S.val * A * S.val⁻¹).charpoly = A.charpoly := by
  exact Matrix.charpoly_units_conj S A

def diagonalExample : Matrix (Fin 2) (Fin 2) ℝ :=
  !![0, 0; 0, 1]

def coordinateChangeExample : Matrix (Fin 2) (Fin 2) ℝ :=
  !![1, 1; 0, 1]

def coordinateChangeInverseExample : Matrix (Fin 2) (Fin 2) ℝ :=
  !![1, -1; 0, 1]

def nonnormalExample : Matrix (Fin 2) (Fin 2) ℝ :=
  !![0, 1; 0, 1]

def secondCoordinateVector : Fin 2 → ℝ :=
  ![0, 1]

/-- The running nonnormal matrix is explicitly similar to the diagonal example. -/
theorem nonnormalExample_similarity :
    coordinateChangeExample * diagonalExample * coordinateChangeInverseExample =
      nonnormalExample := by
  ext i j
  fin_cases i <;> fin_cases j <;>
    norm_num [coordinateChangeExample, diagonalExample,
      coordinateChangeInverseExample, nonnormalExample, Matrix.mul_apply,
      Fin.sum_univ_two]

/-- CFT-01-006: nonunitary similarity changes the output length of a coordinate vector. -/
theorem nonunitary_similarity_changes_output_length_sq :
    (∑ i, (nonnormalExample *ᵥ secondCoordinateVector) i ^ 2) = 2 ∧
      (∑ i, (diagonalExample *ᵥ secondCoordinateVector) i ^ 2) = 1 := by
  constructor <;>
    norm_num [nonnormalExample, diagonalExample, secondCoordinateVector,
      Matrix.mulVec, dotProduct, Fin.sum_univ_two]

namespace Exercises.Chapter01

theorem exercise_01_solution
    {𝕜 V : Type*} [Semiring 𝕜] [AddCommMonoid V] [Module 𝕜 V]
    (T : V →ₗ[𝕜] V) : T 0 = 0 := by
  have hzero : 0 = T 0 := T.map_zero.symm
  exact hzero.symm

/-- Starter: multiply the displayed matrix by each standard coordinate vector,
expanding both two-term dot products. -/
theorem exercise_02_solution :
    ((!![2, -1; 3, 4] : Matrix (Fin 2) (Fin 2) ℤ) *ᵥ ![1, 0]) = ![2, 3] ∧
      ((!![2, -1; 3, 4] : Matrix (Fin 2) (Fin 2) ℤ) *ᵥ ![0, 1]) = ![-1, 4] := by
  constructor <;>
    ext i <;>
    fin_cases i <;>
    norm_num [Matrix.mulVec, dotProduct, Fin.sum_univ_two]

/-- Starter: expand `mulVec` coordinatewise, then commute the scalar factors to
recognize the action as a linear combination of columns. -/
theorem exercise_03_solution
    {𝕜 n : Type*} [CommSemiring 𝕜] [Fintype n]
    (A : Matrix n n 𝕜) (x : n → 𝕜) :
    coordinateAction A x = ∑ j, x j • fun i => A i j := by
  ext i
  simp [coordinateAction, Matrix.mulVec, dotProduct, mul_comm]

/-- Starter: apply similarity invariance separately to the characteristic
polynomial and determinant of the same conjugated matrix. -/
theorem exercise_04_solution
    {𝕜 n : Type*} [CommRing 𝕜] [Fintype n] [DecidableEq n]
    (S : (Matrix n n 𝕜)ˣ) (A : Matrix n n 𝕜) :
    (S.val * A * S.val⁻¹).charpoly = A.charpoly ∧
      Matrix.det (S.val * A * (↑(S⁻¹) : Matrix n n 𝕜)) = Matrix.det A := by
  constructor
  · exact Matrix.charpoly_units_conj S A
  · exact Matrix.det_units_conj S A

/-- Starter: prove the two generic similarity invariants first; then contrast
the explicit nonunitary witness with the orthogonal norm calculation. -/
theorem exercise_05_solution
    {𝕜 n m : Type*} [CommRing 𝕜] [Fintype n] [DecidableEq n]
    [Fintype m] [DecidableEq m]
    (S : (Matrix n n 𝕜)ˣ) (A : Matrix n n 𝕜)
    (U : Matrix m m ℝ) (hU : U.transpose * U = 1) (x : m → ℝ) :
    (S.val * A * S.val⁻¹).charpoly = A.charpoly ∧
      (S.val * A * (↑(S⁻¹) : Matrix n n 𝕜)).trace = A.trace ∧
      coordinateChangeExample * diagonalExample * coordinateChangeInverseExample =
        nonnormalExample ∧
      ((∑ i, (nonnormalExample *ᵥ secondCoordinateVector) i ^ 2) = 2 ∧
        (∑ i, (diagonalExample *ᵥ secondCoordinateVector) i ^ 2) = 1) ∧
      dotProduct (U *ᵥ x) (U *ᵥ x) = dotProduct x x := by
  constructor
  · exact Matrix.charpoly_units_conj S A
  constructor
  · exact Matrix.trace_units_conj S A
  constructor
  · exact nonnormalExample_similarity
  constructor
  · exact nonunitary_similarity_changes_output_length_sq
  calc
    dotProduct (U *ᵥ x) (U *ᵥ x) =
        dotProduct x (U.transpose *ᵥ (U *ᵥ x)) :=
      (Matrix.dotProduct_transpose_mulVec U x (U *ᵥ x)).symm
    _ = dotProduct x ((U.transpose * U) *ᵥ x) := by
      rw [Matrix.mulVec_mulVec]
    _ = dotProduct x x := by rw [hU, Matrix.one_mulVec]

theorem exercise_06_solution :
    (∑ i, (nonnormalExample *ᵥ secondCoordinateVector) i ^ 2) = 2 :=
  nonunitary_similarity_changes_output_length_sq.1

end Exercises.Chapter01

end CrouzeixTextbook.Part01
