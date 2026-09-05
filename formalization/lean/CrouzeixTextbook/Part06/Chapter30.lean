import CrouzeixConjecture.CompletionStatement
import CrouzeixConjecture.CompletionDiagonalization
import CrouzeixConjecture.CompletionGramianBridge

namespace CrouzeixTextbook.Part06
open CrouzeixConjecture
open Filter Set
open scoped BigOperators ComplexConjugate ComplexOrder Matrix Matrix.Norms.L2Operator Topology

noncomputable section

set_option linter.defProp false in
def positive_real_completion := @IsPositiveRealCompletion
/-- The proved auxiliary-basis completion theorem, including the nonempty
finite-dimensional index required by its norm argument. -/
theorem positive_real_completion_statement {n : Type*} [Fintype n] [DecidableEq n]
    [Nonempty n] : PositiveRealCompletionStatement (n := n) :=
  positiveRealCompletionStatement
set_option linter.defProp false in
def completion_gramian_four := @completionP_congruence_eq_gramian_four
set_option linter.defProp false in
def completion_gramian_two := @completionR_congruence_eq_gramian_two
set_option linter.defProp false in
def completion_gramian_difference := @completionX_congruence_eq_gramian_difference
/-- Source positivity survives the self-adjoint inverse-square-root congruence,
and the provider bridge identifies the resulting Gramian expression. -/
theorem completion_gramian_source_positive
    {n : Type*} [Fintype n] [DecidableEq n]
    {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
    (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1)
    (hsource : (4 * completionX G lambda -
      completionX G lambda * G⁻¹ * completionP G lambda -
      completionP G lambda * G⁻¹ * completionX G lambda).PosSemidef) :
    (Hinv * (4 * completionX G lambda -
      completionX G lambda * G⁻¹ * completionP G lambda -
      completionP G lambda * G⁻¹ * completionX G lambda) * Hinv).PosSemidef ∧
    (4 * (gramian 2 (completionSimilarity H Hinv lambda) -
        gramian 4 (completionSimilarity H Hinv lambda)) -
      (gramian 2 (completionSimilarity H Hinv lambda) -
        gramian 4 (completionSimilarity H Hinv lambda)) *
          gramian 4 (completionSimilarity H Hinv lambda) -
      gramian 4 (completionSimilarity H Hinv lambda) *
        (gramian 2 (completionSimilarity H Hinv lambda) -
          gramian 4 (completionSimilarity H Hinv lambda))).PosSemidef := by
  have hcongr := hsource.conjTranspose_mul_mul_same Hinv
  rw [hsqrt.hHinv_selfAdjoint] at hcongr
  exact ⟨hcongr,
    completion_gramian_expression_posSemidef_of_source hsqrt lambda hlambda hsource⟩

namespace Exercises.Chapter30

variable {n : Type*} [Fintype n] [DecidableEq n]

/-- Starter: unfold the completion predicate, then use nested rcases to name
the analytic, normalization, positivity, and algebra-defect clauses. -/
theorem exercise_01_solution
    (B T : SquareMatrix n) (H : ℂ → SquareMatrix n)
    (hcompletion : IsPositiveRealCompletion B T H) :
    AnalyticOnNhd ℂ H unitDisk ∧ H 0 = 1 ∧
      (∀ z ∈ unitDisk, IsPositiveMatrix (rePart (H z))) ∧
      ∀ z ∈ unitDisk, H z - (1 - z • T)⁻¹ ∈ generatedAlgebra Bᴴ := by
  rcases hcompletion with ⟨hanalytic, hzero, hpositive, hdefect⟩
  exact ⟨hanalytic, hzero, hpositive, hdefect⟩

set_option linter.unusedVariables false in
/-- The same change of basis sends two diagonal matrices to B and T, so their
diagonal commutation survives under the algebra equivalence. -/
theorem exercise_02_solution
    (B T : SquareMatrix n) (H : ℂ → SquareMatrix n)
    (hB : SimpleDiagonalization B) (lambda : n → ℂ)
    (hT : T = innerConjugation hB.changeBasis (Matrix.diagonal lambda))
    (hlambda : ∀ i, ‖lambda i‖ ≤ 1)
    (hcompletion : IsPositiveRealCompletion B T H) : Commute B T := by
  have hdiag :
      Commute (Matrix.diagonal hB.eigenvalues) (Matrix.diagonal lambda) :=
    Matrix.commute_diagonal hB.eigenvalues lambda
  rw [hB.eq_conjugate, hT]
  exact hdiag.map (innerConjugation hB.changeBasis)

/-- Expand the geometric series entrywise before carrying the sum through the
two-sided congruence. -/
theorem exercise_03_solution
    {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
    (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1) (i j : n) :
    completionP G lambda i j = G i j / (1 - conj (lambda i) * lambda j / 4) ∧
      Hinv * completionP G lambda * Hinv =
        gramian 4 (completionSimilarity H Hinv lambda) := by
  constructor
  · rfl
  · have hs : Summable (completionDiagonalGramianTerm 4 G lambda) :=
      summable_completionDiagonalGramianTerm hsqrt lambda hlambda (by norm_num)
    rw [← tsum_completionDiagonalGramianTerm_four_eq_completionP hsqrt lambda hlambda]
    calc
      Hinv * (∑' k : ℕ, completionDiagonalGramianTerm 4 G lambda k) * Hinv =
          ∑' k : ℕ, Hinv * completionDiagonalGramianTerm 4 G lambda k * Hinv := by
        rw [← hs.tsum_mul_left Hinv, ← (hs.mul_left Hinv).tsum_mul_right Hinv]
      _ = ∑' k : ℕ, gramianTerm 4 (completionSimilarity H Hinv lambda) k := by
        exact tsum_congr fun k ↦
          (gramianTerm_completionSimilarity_eq hsqrt lambda 4 k).symm
      _ = gramian 4 (completionSimilarity H Hinv lambda) := rfl

/-- The denominator-two calculation is the same termwise congruence with the
weight changed from four to two. -/
theorem exercise_04_solution
    {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
    (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1) (i j : n) :
    completionR G lambda i j = G i j / (1 - conj (lambda i) * lambda j / 2) ∧
      Hinv * completionR G lambda * Hinv =
        gramian 2 (completionSimilarity H Hinv lambda) := by
  constructor
  · rfl
  · have hs : Summable (completionDiagonalGramianTerm 2 G lambda) :=
      summable_completionDiagonalGramianTerm hsqrt lambda hlambda (by norm_num)
    rw [← tsum_completionDiagonalGramianTerm_two_eq_completionR hsqrt lambda hlambda]
    calc
      Hinv * (∑' k : ℕ, completionDiagonalGramianTerm 2 G lambda k) * Hinv =
          ∑' k : ℕ, Hinv * completionDiagonalGramianTerm 2 G lambda k * Hinv := by
        rw [← hs.tsum_mul_left Hinv, ← (hs.mul_left Hinv).tsum_mul_right Hinv]
      _ = ∑' k : ℕ, gramianTerm 2 (completionSimilarity H Hinv lambda) k := by
        exact tsum_congr fun k ↦
          (gramianTerm_completionSimilarity_eq hsqrt lambda 2 k).symm
      _ = gramian 2 (completionSimilarity H Hinv lambda) := rfl

/-- Distribute the congruence over R minus P, then substitute the two
previously proved weighted-Gramian identities. -/
theorem exercise_05_solution
    {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
    (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1) :
    completionX G lambda = completionR G lambda - completionP G lambda ∧
      Hinv * completionX G lambda * Hinv =
        gramian 2 (completionSimilarity H Hinv lambda) -
          gramian 4 (completionSimilarity H Hinv lambda) := by
  constructor
  · rfl
  · have hs2 : Summable (completionDiagonalGramianTerm 2 G lambda) :=
      summable_completionDiagonalGramianTerm hsqrt lambda hlambda (by norm_num)
    have hs4 : Summable (completionDiagonalGramianTerm 4 G lambda) :=
      summable_completionDiagonalGramianTerm hsqrt lambda hlambda (by norm_num)
    have hR :
        Hinv * completionR G lambda * Hinv =
          gramian 2 (completionSimilarity H Hinv lambda) := by
      rw [← tsum_completionDiagonalGramianTerm_two_eq_completionR hsqrt lambda hlambda]
      calc
        Hinv * (∑' k : ℕ, completionDiagonalGramianTerm 2 G lambda k) * Hinv =
            ∑' k : ℕ, Hinv * completionDiagonalGramianTerm 2 G lambda k * Hinv := by
          rw [← hs2.tsum_mul_left Hinv,
            ← (hs2.mul_left Hinv).tsum_mul_right Hinv]
        _ = ∑' k : ℕ, gramianTerm 2 (completionSimilarity H Hinv lambda) k := by
          exact tsum_congr fun k ↦
            (gramianTerm_completionSimilarity_eq hsqrt lambda 2 k).symm
        _ = gramian 2 (completionSimilarity H Hinv lambda) := rfl
    have hP :
        Hinv * completionP G lambda * Hinv =
          gramian 4 (completionSimilarity H Hinv lambda) := by
      rw [← tsum_completionDiagonalGramianTerm_four_eq_completionP hsqrt lambda hlambda]
      calc
        Hinv * (∑' k : ℕ, completionDiagonalGramianTerm 4 G lambda k) * Hinv =
            ∑' k : ℕ, Hinv * completionDiagonalGramianTerm 4 G lambda k * Hinv := by
          rw [← hs4.tsum_mul_left Hinv,
            ← (hs4.mul_left Hinv).tsum_mul_right Hinv]
        _ = ∑' k : ℕ, gramianTerm 4 (completionSimilarity H Hinv lambda) k := by
          exact tsum_congr fun k ↦
            (gramianTerm_completionSimilarity_eq hsqrt lambda 4 k).symm
        _ = gramian 4 (completionSimilarity H Hinv lambda) := rfl
    rw [completionX, mul_sub, sub_mul, hR, hP]

/-- Congruence by the self-adjoint inverse square root preserves source
positivity; the algebraic bridge then identifies the congruenced matrix. -/
theorem exercise_06_solution
    {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
    (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1)
    (hsource : (4 * completionX G lambda -
      completionX G lambda * G⁻¹ * completionP G lambda -
      completionP G lambda * G⁻¹ * completionX G lambda).PosSemidef) :
    (Hinv * (4 * completionX G lambda -
      completionX G lambda * G⁻¹ * completionP G lambda -
      completionP G lambda * G⁻¹ * completionX G lambda) * Hinv).PosSemidef ∧
    Hinv * (4 * completionX G lambda -
      completionX G lambda * G⁻¹ * completionP G lambda -
      completionP G lambda * G⁻¹ * completionX G lambda) * Hinv =
      4 * (gramian 2 (completionSimilarity H Hinv lambda) -
        gramian 4 (completionSimilarity H Hinv lambda)) -
      (gramian 2 (completionSimilarity H Hinv lambda) -
        gramian 4 (completionSimilarity H Hinv lambda)) *
          gramian 4 (completionSimilarity H Hinv lambda) -
      gramian 4 (completionSimilarity H Hinv lambda) *
        (gramian 2 (completionSimilarity H Hinv lambda) -
          gramian 4 (completionSimilarity H Hinv lambda)) := by
  have hcongr := hsource.conjTranspose_mul_mul_same Hinv
  rw [hsqrt.hHinv_selfAdjoint] at hcongr
  refine ⟨hcongr, ?_⟩
  rw [completion_congruence_identity Hinv G⁻¹ (completionP G lambda)
    (completionX G lambda) (completionGram_inverse_eq hsqrt)]
  rw [completionX_congruence_eq_gramian_difference hsqrt lambda hlambda,
    completionP_congruence_eq_gramian_four hsqrt lambda hlambda]

end Exercises.Chapter30

end
end CrouzeixTextbook.Part06
