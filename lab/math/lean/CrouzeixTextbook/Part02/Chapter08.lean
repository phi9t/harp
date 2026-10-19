import CrouzeixConjecture.CompletionSquareRoot
import CrouzeixConjecture.Positivity
import CrouzeixTextbook.Part02.Chapter07

namespace CrouzeixTextbook.Part02

open CrouzeixConjecture
open scoped Matrix ComplexOrder

set_option linter.defProp false

section RealPositivity

variable {n : ℕ}

/-- CFT-08-001: a symmetric real matrix whose quadratic form is positive on every
nonzero vector is positive at any explicitly supplied nonzero vector. The content
is the instantiation, not a new inequality. -/
theorem positive_definite_quadratic_positive
    (A : Matrix (Fin n) (Fin n) ℝ) (_hSymm : A.IsSymm)
    (hPositive : ∀ y : Fin n → ℝ, y ≠ 0 → 0 < y ⬝ᵥ (A *ᵥ y))
    {x : Fin n → ℝ} (hx : x ≠ 0) :
    0 < x ⬝ᵥ (A *ᵥ x) := by
  have h := hPositive x hx
  exact h

/-- The real Gram matrix of a family of columns, written through the transpose. -/
theorem real_gram_quadratic_form (S : Matrix (Fin n) (Fin n) ℝ) (x : Fin n → ℝ) :
    x ⬝ᵥ ((S.transpose * S) *ᵥ x) = (S *ᵥ x) ⬝ᵥ (S *ᵥ x) := by
  rw [← Matrix.mulVec_mulVec, Matrix.dotProduct_mulVec, Matrix.vecMul_transpose]

/-- Every real Gram matrix is positive semidefinite, because its quadratic form is a
squared length. -/
theorem real_gram_nonneg (S : Matrix (Fin n) (Fin n) ℝ) (x : Fin n → ℝ) :
    0 ≤ x ⬝ᵥ ((S.transpose * S) *ᵥ x) := by
  rw [real_gram_quadratic_form]
  exact real_inner_self_nonneg _

/-- Positive semidefiniteness is strictly weaker than positive definiteness: the real
Gram matrix of a singular family has a nonzero null vector. -/
theorem real_gram_semidefinite_not_definite :
    ∃ (S : Matrix (Fin 2) (Fin 2) ℝ) (x : Fin 2 → ℝ),
      x ≠ 0 ∧ x ⬝ᵥ ((S.transpose * S) *ᵥ x) = 0 := by
  refine ⟨!![1, 1; 0, 0], ![1, -1], ?_, ?_⟩
  · intro h
    have := congrFun h 0
    simp at this
  · rw [real_gram_quadratic_form]
    simp [Matrix.mulVec, dotProduct, Fin.sum_univ_two]

/-- Entrywise positivity is a different condition again: this matrix has strictly
positive entries and an indefinite quadratic form. -/
theorem entrywise_positive_not_semidefinite :
    (∀ i j, 0 < (!![1, 2; 2, 1] : Matrix (Fin 2) (Fin 2) ℝ) i j) ∧
      (![1, -1] : Fin 2 → ℝ) ⬝ᵥ ((!![1, 2; 2, 1] : Matrix (Fin 2) (Fin 2) ℝ) *ᵥ ![1, -1])
        < 0 := by
  constructor
  · intro i j
    fin_cases i <;> fin_cases j <;> norm_num
  · simp [Matrix.mulVec, dotProduct, Fin.sum_univ_two]

end RealPositivity

section ComplexPositivity

/-- CFT-08-002: every Gram matrix is positive semidefinite. -/
def gram_matrix_positive := @completionGramMatrix_posSemidef

/-- CFT-08-003: an invertible generator has an invertible Gram matrix. -/
def invertible_gram_matrix_invertible := @completionGramMatrix_isUnit

/-- CFT-08-004: an invertible generator has a positive definite Gram matrix. -/
def invertible_gram_matrix_positive_definite := @completionGramMatrix_posDef

/-- CFT-08-005: congruence preserves positive semidefiniteness. -/
def positivity_preserved_by_congruence := @posSemidef_congruence

/-- CFT-08-006: an invertible generator supplies complete positive square-root data. -/
def positive_square_root_data := @completionSquareRootData_of_isUnit

variable {n : Type*} [Fintype n] [DecidableEq n]

omit [DecidableEq n] in
/-- Unfolding the Gram matrix of this development. -/
theorem completionGramMatrix_eq (S : SquareMatrix n) :
    completionGramMatrix S = Sᴴ * S := rfl

/-- The square root supplied by CFT-08-006 really squares back to the Gram matrix. -/
theorem completion_square_root_squares (S : SquareMatrix n) (hS : IsUnit S) :
    completionPositiveSquareRoot S * completionPositiveSquareRoot S
      = completionGramMatrix S :=
  (completionSquareRootData_of_isUnit S hS).hH_mul_H

/-- The square root is self-adjoint, which is what makes it *the* positive root
rather than an arbitrary factorization. -/
theorem completion_square_root_selfAdjoint (S : SquareMatrix n) (hS : IsUnit S) :
    (completionPositiveSquareRoot S)ᴴ = completionPositiveSquareRoot S :=
  (completionSquareRootData_of_isUnit S hS).hH_selfAdjoint

/-- The supplied inverse really inverts the square root on both sides. -/
theorem completion_square_root_inverse (S : SquareMatrix n) (hS : IsUnit S) :
    completionPositiveSquareRoot S * completionPositiveInvSquareRoot S = 1 ∧
      completionPositiveInvSquareRoot S * completionPositiveSquareRoot S = 1 :=
  ⟨(completionSquareRootData_of_isUnit S hS).hH_mul_Hinv,
    (completionSquareRootData_of_isUnit S hS).hHinv_mul_H⟩

end ComplexPositivity

namespace Exercises.Chapter08

/-- CFT-08-E01. -/
theorem exercise_01_solution (x : Fin 2 → ℝ) (hx : x ≠ 0) :
    0 < x ⬝ᵥ ((1 : Matrix (Fin 2) (Fin 2) ℝ) *ᵥ x) := by
  refine positive_definite_quadratic_positive 1 Matrix.isSymm_one ?_ hx
  intro y hy
  rw [Matrix.one_mulVec]
  have hnn := real_inner_self_nonneg y
  have hne : y ⬝ᵥ y ≠ 0 := fun h => hy ((real_inner_self_eq_zero_iff y).mp h)
  exact lt_of_le_of_ne hnn (Ne.symm hne)

/-- CFT-08-E02. -/
theorem exercise_02_solution (S : Matrix (Fin 2) (Fin 2) ℝ) (x : Fin 2 → ℝ) :
    x ⬝ᵥ ((S.transpose * S) *ᵥ x) = (S *ᵥ x) ⬝ᵥ (S *ᵥ x) ∧
      0 ≤ x ⬝ᵥ ((S.transpose * S) *ᵥ x) :=
  ⟨real_gram_quadratic_form S x, real_gram_nonneg S x⟩

/-- CFT-08-E03. -/
theorem exercise_03_solution {n : Type*} [Fintype n] [DecidableEq n]
    (S : SquareMatrix n) (hS : IsUnit S) :
    (completionGramMatrix S).PosSemidef ∧ IsUnit (completionGramMatrix S) ∧
      (completionGramMatrix S).PosDef :=
  ⟨completionGramMatrix_posSemidef S, completionGramMatrix_isUnit S hS,
    completionGramMatrix_posDef S hS⟩

/-- CFT-08-E04. -/
theorem exercise_04_solution {n : Type*} [Fintype n] [DecidableEq n]
    {A : SquareMatrix n} (hA : A.PosSemidef) (S T : SquareMatrix n) :
    (Sᴴ * A * S).PosSemidef ∧ ((S * T)ᴴ * A * (S * T)).PosSemidef := by
  refine ⟨posSemidef_congruence hA S, ?_⟩
  exact posSemidef_congruence hA (S * T)

/-- CFT-08-E05. -/
theorem exercise_05_solution :
    (∃ (S : Matrix (Fin 2) (Fin 2) ℝ) (x : Fin 2 → ℝ),
        x ≠ 0 ∧ x ⬝ᵥ ((S.transpose * S) *ᵥ x) = 0) ∧
      ((∀ i j, 0 < (!![1, 2; 2, 1] : Matrix (Fin 2) (Fin 2) ℝ) i j) ∧
        (![1, -1] : Fin 2 → ℝ) ⬝ᵥ
            ((!![1, 2; 2, 1] : Matrix (Fin 2) (Fin 2) ℝ) *ᵥ ![1, -1]) < 0) :=
  ⟨real_gram_semidefinite_not_definite, entrywise_positive_not_semidefinite⟩

/-- CFT-08-E06. -/
theorem exercise_06_solution {n : Type*} [Fintype n] [DecidableEq n]
    (S : SquareMatrix n) (hS : IsUnit S) :
    completionPositiveSquareRoot S * completionPositiveSquareRoot S
        = completionGramMatrix S ∧
      (completionPositiveSquareRoot S)ᴴ = completionPositiveSquareRoot S ∧
      completionPositiveSquareRoot S * completionPositiveInvSquareRoot S = 1 :=
  ⟨completion_square_root_squares S hS, completion_square_root_selfAdjoint S hS,
    (completion_square_root_inverse S hS).1⟩

end Exercises.Chapter08

end CrouzeixTextbook.Part02
