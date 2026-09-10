import CrouzeixConjecture.CompletionSquareRoot
import CrouzeixConjecture.Positivity
import CrouzeixTextbook.Part02.Chapter08

namespace CrouzeixTextbook.Part02

open CrouzeixConjecture
open scoped Matrix Matrix.Norms.L2Operator ComplexOrder

set_option linter.defProp false

/-- CFT-09-001: the matrix norm of this development is the induced operator norm.
This card re-exports the same provider as CFT-07-005; see the chapter prose. -/
def induced_matrix_norm_identity := @matrix_norm_eq_euclidean_operator_norm

/-- CFT-09-002: the polar factor of an invertible matrix is unitary. -/
def polar_factor_is_unitary := @completionPolarUnitary_mem_unitaryGroup

/-- CFT-09-003: the polar similarity preserves the matrix norm. -/
def polar_similarity_norm_transfer :=
  @completionDiagonalizableMatrix_norm_eq_completionSimilarity_norm

/-- CFT-09-004: the same transfer read on induced operator norms. -/
def polar_operator_norm_transfer :=
  @completionDiagonalizableMatrix_euclideanOperator_norm_eq

/-- CFT-09-005: a quadratic bound forces the operator norm to be at most two. -/
def quadratic_bound_implies_norm_two :=
  @matrix_norm_le_two_of_four_sub_conjTranspose_mul_self_posSemidef

/-- CFT-09-006: the triangle inequality, the kernel of every norm estimate below. -/
theorem norm_triangle_kernel {E : Type*} [SeminormedAddGroup E] (x y : E) :
    ‖x + y‖ ≤ ‖x‖ + ‖y‖ := by
  have h := norm_add_le x y
  exact h

section InducedNorm

variable {n : Type*} [Fintype n] [DecidableEq n]

/-- The defining bound of an induced norm: it dominates every vector's stretching. -/
theorem matrix_norm_mulVec_bound (A : SquareMatrix n) (x : EuclideanVector n) :
    ‖euclideanOperator A x‖ ≤ ‖A‖ * ‖x‖ := by
  rw [matrix_norm_eq_euclidean_operator_norm]
  exact (euclideanOperator A).le_opNorm x

/-- Conversely the induced norm is the least such bound. -/
theorem matrix_norm_le_of_bound (A : SquareMatrix n) {C : ℝ} (hC : 0 ≤ C)
    (h : ∀ x : EuclideanVector n, ‖euclideanOperator A x‖ ≤ C * ‖x‖) :
    ‖A‖ ≤ C := by
  rw [matrix_norm_eq_euclidean_operator_norm]
  exact (euclideanOperator A).opNorm_le_bound hC h

/-- The norm is submultiplicative, which is what makes power bounds meaningful. -/
theorem matrix_norm_mul_le (A B : SquareMatrix n) : ‖A * B‖ ≤ ‖A‖ * ‖B‖ :=
  norm_mul_le A B

/-- The C-star identity. This is the bridge from the operator norm to the singular
values: the norm squared is the norm of the positive matrix whose eigenvalues are
the squared singular values. -/
theorem matrix_norm_conjTranspose_mul_self (A : SquareMatrix n) :
    ‖Aᴴ * A‖ = ‖A‖ * ‖A‖ :=
  CStarRing.norm_star_mul_self

/-- Unitary factors on either side leave the norm unchanged. -/
theorem matrix_norm_unitary_invariant {U : SquareMatrix n} (hU : U ∈ unitary (SquareMatrix n))
    (A : SquareMatrix n) : ‖U * A‖ = ‖A‖ ∧ ‖A * U‖ = ‖A‖ :=
  ⟨CStarRing.norm_mem_unitary_mul A hU, CStarRing.norm_mul_mem_unitary A hU⟩

/-- Rank deficiency in the norm language: a singular matrix kills a unit vector, so
its smallest singular value is zero. -/
theorem singular_has_null_vector (A : SquareMatrix n) (hA : A.det = 0) :
    ∃ x : n → ℂ, x ≠ 0 ∧ A *ᵥ x = 0 :=
  Matrix.exists_mulVec_eq_zero_iff.mpr hA

end InducedNorm

section NonnormalAmplification

/-- The running family at eigenvalue zero. It is nilpotent, so every eigenvalue is
zero and the spectral radius vanishes. -/
theorem shear_nilpotent (α : ℂ) :
    (!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ) *
      (!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ) = 0 := by
  ext i j
  fin_cases i <;> fin_cases j <;>
    simp [Matrix.mul_apply, Fin.sum_univ_two]

/-- Its determinant vanishes, so it is singular. -/
theorem shear_det_zero (α : ℂ) :
    (!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ).det = 0 := by
  simp [Matrix.det_fin_two_of]

/-- The characteristic polynomial of the shear is `X ^ 2`. Zero is therefore its
only characteristic root, which is the compiled content of the "every eigenvalue
vanishes" step in this chapter's central counterexample. -/
theorem shear_charpoly (α : ℂ) :
    (!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ).charpoly = Polynomial.X ^ 2 := by
  rw [Matrix.charpoly_fin_two, Matrix.trace_fin_two_of, Matrix.det_fin_two_of]
  simp

/-- Zero is the only eigenvalue of the shear, so its spectral radius is zero while
its norm is `‖α‖`. -/
theorem shear_spectrum_eq_zero (α μ : ℂ) :
    μ ∈ spectrum ℂ (!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ) ↔ μ = 0 := by
  rw [Matrix.mem_spectrum_iff_isRoot_charpoly, shear_charpoly]
  simp [Polynomial.IsRoot]

/-- The Gram matrix of the shear is diagonal with a single nonzero entry. -/
theorem shear_conjTranspose_mul_self (α : ℂ) :
    (!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ)ᴴ *
        (!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ)
      = Matrix.diagonal ![0, (starRingEnd ℂ) α * α] := by
  ext i j
  fin_cases i <;> fin_cases j <;>
    simp [Matrix.conjTranspose_apply, Matrix.mul_apply, Fin.sum_univ_two]

/-- The supremum norm of that diagonal family is the squared modulus. -/
theorem shear_diagonal_family_norm (α : ℂ) :
    ‖(![0, (starRingEnd ℂ) α * α] : Fin 2 → ℂ)‖ = ‖α‖ * ‖α‖ := by
  have hentry : ‖(starRingEnd ℂ) α * α‖ = ‖α‖ * ‖α‖ := by
    rw [norm_mul, RCLike.norm_conj]
  refine le_antisymm ?_ ?_
  · refine (pi_norm_le_iff_of_nonneg (by positivity)).2 fun i => ?_
    fin_cases i
    · simp
      positivity
    · simp
  · have h := norm_le_pi_norm (![0, (starRingEnd ℂ) α * α] : Fin 2 → ℂ) 1
    simpa [hentry] using h

/-- The exact induced norm of the shear is the shear parameter. Every eigenvalue is
zero, so the spectral radius is zero: the two measurements disagree by `‖α‖`. -/
theorem shear_norm_eq (α : ℂ) :
    ‖(!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ)‖ = ‖α‖ := by
  set A : Matrix (Fin 2) (Fin 2) ℂ := !![0, α; 0, 0] with hA
  have hsq : ‖A‖ * ‖A‖ = ‖α‖ * ‖α‖ := by
    rw [← CStarRing.norm_star_mul_self]
    show ‖Aᴴ * A‖ = ‖α‖ * ‖α‖
    rw [hA, shear_conjTranspose_mul_self, Matrix.l2_opNorm_diagonal,
      shear_diagonal_family_norm]
  nlinarith [norm_nonneg A, norm_nonneg α, hsq]

end NonnormalAmplification

namespace Exercises.Chapter09

/-- CFT-09-E01. -/
theorem exercise_01_solution {n : Type*} [Fintype n] [DecidableEq n]
    (A : SquareMatrix n) (x : EuclideanVector n) :
    ‖A‖ = ‖euclideanOperator A‖ ∧ ‖euclideanOperator A x‖ ≤ ‖A‖ * ‖x‖ :=
  ⟨matrix_norm_eq_euclidean_operator_norm A, matrix_norm_mulVec_bound A x⟩

/-- CFT-09-E02. -/
theorem exercise_02_solution {n : Type*} [Fintype n] [DecidableEq n]
    (A B : SquareMatrix n) :
    ‖A * B‖ ≤ ‖A‖ * ‖B‖ ∧ ‖Aᴴ * A‖ = ‖A‖ * ‖A‖ :=
  ⟨matrix_norm_mul_le A B, matrix_norm_conjTranspose_mul_self A⟩

/-- CFT-09-E03. -/
theorem exercise_03_solution {n : Type*} [Fintype n] [DecidableEq n]
    (S : SquareMatrix n) (hS : IsUnit S) (lambda : n → ℂ) :
    completionPolarUnitary S ∈ Matrix.unitaryGroup n ℂ ∧
      ‖completionDiagonalizableMatrix S lambda‖ =
        ‖completionSimilarity (completionPositiveSquareRoot S)
          (completionPositiveInvSquareRoot S) lambda‖ :=
  ⟨completionPolarUnitary_mem_unitaryGroup S hS,
    completionDiagonalizableMatrix_norm_eq_completionSimilarity_norm S hS lambda⟩

/-- CFT-09-E04. -/
theorem exercise_04_solution {n : Type*} [Fintype n] [DecidableEq n]
    {U : SquareMatrix n} (hU : U ∈ unitary (SquareMatrix n)) (A : SquareMatrix n) :
    ‖U * A‖ = ‖A‖ ∧ ‖A * U‖ = ‖A‖ ∧ ‖U * A * Uᴴ‖ = ‖A‖ := by
  have hUstar : Uᴴ ∈ unitary (SquareMatrix n) := Unitary.star_mem hU
  refine ⟨CStarRing.norm_mem_unitary_mul A hU, CStarRing.norm_mul_mem_unitary A hU, ?_⟩
  rw [CStarRing.norm_mul_mem_unitary _ hUstar, CStarRing.norm_mem_unitary_mul A hU]

/-- CFT-09-E05. -/
theorem exercise_05_solution (α : ℂ) :
    (!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ).det = 0 ∧
      (!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ) *
          (!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ) = 0 ∧
      ‖(!![0, α; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ)‖ = ‖α‖ :=
  ⟨shear_det_zero α, shear_nilpotent α, shear_norm_eq α⟩

/-- CFT-09-E06. -/
theorem exercise_06_solution {n : Type*} [Fintype n] [DecidableEq n]
    (C : SquareMatrix n) (hC : ((4 : ℂ) • (1 : SquareMatrix n) - Cᴴ * C).PosSemidef)
    (x y : SquareMatrix n) :
    ‖C‖ ≤ 2 ∧ ‖x + y‖ ≤ ‖x‖ + ‖y‖ :=
  ⟨matrix_norm_le_two_of_four_sub_conjTranspose_mul_self_posSemidef C hC,
    norm_triangle_kernel x y⟩

end Exercises.Chapter09

end CrouzeixTextbook.Part02
