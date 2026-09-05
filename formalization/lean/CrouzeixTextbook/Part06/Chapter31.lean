import CrouzeixConjecture.CompletionKernelModel

namespace CrouzeixTextbook.Part06
open CrouzeixConjecture
open scoped BigOperators ComplexConjugate ComplexOrder Matrix

noncomputable section

set_option linter.defProp false in
def completion_kernel_model := @completionKernelModel

set_option linter.defProp false in
/-- Compatibility checkpoint for the forward implication.  CFT-31-002 uses
the reverse theorem below. -/
def completion_kernel_at_zero := @completionKernelModel_zero

/-- Algebraic bridge for the reverse normalization implication.  The provider
`completionKernelModel_zero` proves the other direction. -/
theorem completion_kernel_normalization_forces_correction_zero_bridge
    {n : Type*} [Fintype n] [DecidableEq n]
    (G : SquareMatrix n) (hGunit : IsUnit G) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (hnormalized : completionKernelModel G lambda d 0 = G) :
    d 0 = 0 := by
  have hdet : IsUnit G.det := G.isUnit_iff_isUnit_det.mp hGunit
  have hcorrection : completionDiagonalCorrection d 0 * G = 0 := by
    have hmodel : G + completionDiagonalCorrection d 0 * G = G := by
      simpa [completionKernelModel] using hnormalized
    simpa using hmodel
  have hdiagonal : completionDiagonalCorrection d 0 = 0 := by
    calc
      completionDiagonalCorrection d 0 =
          (completionDiagonalCorrection d 0 * G) * G⁻¹ := by
        rw [Matrix.mul_assoc, G.mul_nonsing_inv hdet, Matrix.mul_one]
      _ = 0 := by rw [hcorrection, Matrix.zero_mul]
  funext i
  have hii := congrFun₂ hdiagonal i i
  simpa [completionDiagonalCorrection] using hii

/-- If the kernel is normalized at the origin and `G` is invertible, then the
diagonal correction vanishes at the origin. -/
theorem completion_kernel_normalization_forces_correction_zero
    {n : Type*} [Fintype n] [DecidableEq n]
    (G : SquareMatrix n) (hGunit : IsUnit G) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (hnormalized : completionKernelModel G lambda d 0 = G) :
    d 0 = 0 := by
  have hzero := completion_kernel_normalization_forces_correction_zero_bridge
    G hGunit lambda d hnormalized
  rw [hzero]

set_option linter.defProp false in
def sample_origin_quadratic_identity := @completionResolventKernel_sampling_quadratic_eq
set_option linter.defProp false in
def correction_sampling_identity := @completionCorrectionKernel_sampling_quadratic_eq
set_option linter.defProp false in
def correction_sampling_cancels := @completionCorrectionKernel_sampling_eq_zero
set_option linter.defProp false in
def kernel_positivity_implies_X := @completion_X_inequality_of_positiveKernel

namespace Exercises.Chapter31

variable {n : Type*} [Fintype n]

/-- A sparse vector selects the `j`th column.  Applying that fact to the
kernel model reduces the requested column entry to the scalar formula. -/
theorem exercise_01_solution
    [DecidableEq n]
    (G : SquareMatrix n) (lambda : n → ℂ) (d : ℂ → n → ℂ)
    (z : ℂ) (i j : n) :
    (completionKernelModel G lambda d z *ᵥ
        completionSparseVector (fun _ ↦ (1 : ℂ)) j) i =
      completionKernelModel G lambda d z i j ∧
    completionKernelModel G lambda d z i j =
      G i j * (1 - z * lambda j)⁻¹ + d z i * G i j := by
  have hcolumn := mulVec_completionSparseVector
    (completionKernelModel G lambda d z) (fun _ ↦ (1 : ℂ)) j i
  have hselected :
      (completionKernelModel G lambda d z *ᵥ
          completionSparseVector (fun _ ↦ (1 : ℂ)) j) i =
        completionKernelModel G lambda d z i j := by
    simpa using hcolumn
  refine ⟨hselected, ?_⟩
  calc
    completionKernelModel G lambda d z i j =
        (completionKernelModel G lambda d z *ᵥ
          completionSparseVector (fun _ ↦ (1 : ℂ)) j) i := hselected.symm
    _ = G i j * (1 - z * lambda j)⁻¹ + d z i * G i j :=
      by
        simp [completionKernelModel, completionDiagonalCorrection, Matrix.mulVec,
          dotProduct, Matrix.mul_apply, Matrix.diagonal, completionSparseVector,
          mul_ite]

/-- Repeat the reverse normalization algebra without invoking either the
public theorem or the forward provider theorem. -/
theorem exercise_02_solution
    [DecidableEq n]
    (G : SquareMatrix n) (hGunit : IsUnit G) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (hnormalized : completionKernelModel G lambda d 0 = G) :
    completionDiagonalCorrection d 0 = 0 ∧ d 0 = 0 := by
  have hdet : IsUnit G.det := G.isUnit_iff_isUnit_det.mp hGunit
  have hcorrection : completionDiagonalCorrection d 0 * G = 0 := by
    have hmodel : G + completionDiagonalCorrection d 0 * G = G := by
      simpa [completionKernelModel] using hnormalized
    simpa using hmodel
  have hdiagonal : completionDiagonalCorrection d 0 = 0 := by
    calc
      completionDiagonalCorrection d 0 =
          (completionDiagonalCorrection d 0 * G) * G⁻¹ := by
        rw [Matrix.mul_assoc, G.mul_nonsing_inv hdet, Matrix.mul_one]
      _ = 0 := by rw [hcorrection, Matrix.zero_mul]
  refine ⟨hdiagonal, ?_⟩
  funext i
  have hii := congrFun₂ hdiagonal i i
  simpa [completionDiagonalCorrection] using hii

/-- Prove each sampled block separately, keeping the four roles visible. -/
theorem exercise_03_solution
    [DecidableEq n]
    {G : SquareMatrix n} (hG : G.IsHermitian) (lambda : n → ℂ)
    (hlambda : ∀ i, ‖lambda i‖ ≤ 1) :
    (∀ i j, matrixHerglotzKernel (completionResolventModel G lambda)
      (conj (lambda i) / 2) (conj (lambda j) / 2) i j =
        (4 • completionR G lambda - 2 • completionP G lambda) i j) ∧
    (∀ i j, matrixHerglotzKernel (completionResolventModel G lambda)
      (conj (lambda i) / 2) 0 i j = (G + completionR G lambda) i j) ∧
    (∀ i j, matrixHerglotzKernel (completionResolventModel G lambda)
      0 (conj (lambda j) / 2) i j = (G + completionR G lambda) i j) ∧
    ∀ i j, matrixHerglotzKernel (completionResolventModel G lambda) 0 0 i j =
      (2 • G) i j := by
  refine ⟨?_, ?_, ?_, ?_⟩
  · intro i j
    have hfour : (4 : ℕ) • completionR G lambda =
        (4 : ℂ) • completionR G lambda := by
      ext a b
      norm_num [Matrix.smul_apply]
    have htwo : (2 : ℕ) • completionP G lambda =
        (2 : ℂ) • completionP G lambda := by
      ext a b
      norm_num [Matrix.smul_apply]
    rw [hfour, htwo]
    have h := completionResolventKernel_sample_sample_apply hG lambda hlambda i j
    exact h
  · intro i j
    exact completionResolventKernel_sample_zero_apply hG lambda i j
  · intro i j
    exact completionResolventKernel_zero_sample_apply hG lambda i j
  · intro i j
    have htwo : (2 : ℕ) • G = (2 : ℂ) • G := by
      ext a b
      norm_num [Matrix.smul_apply]
    rw [htwo]
    have h := completionResolventKernel_zero_zero_apply hG lambda i j
    exact h

/-- Expand the correction sampling sum into the half-contribution and its
conjugate. -/
theorem exercise_04_solution
    [DecidableEq n]
    {G : SquareMatrix n} (hG : G.IsHermitian) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (hd0 : d 0 = 0) (u : n → ℂ) :
    let theta := completionUnknownHalfContribution G lambda d u
    let correctionSum := ∑ a, ∑ b,
      star (completionSampleVector G lambda u a) ⬝ᵥ
        (matrixHerglotzKernel (completionCorrectionModel G d)
          (completionSamplePoint lambda a) (completionSamplePoint lambda b) *ᵥ
            completionSampleVector G lambda u b)
    correctionSum = theta + conj theta := by
  dsimp
  let Delta := completionSampleDiagonal d lambda
  let P := completionP G lambda
  let v := completionV G P u
  have hP : P.IsHermitian := completionP_isHermitian hG lambda
  have hhalf : completionUnknownHalfContribution G lambda d u =
      star u ⬝ᵥ ((Delta * G) *ᵥ v) +
        star u ⬝ᵥ ((Delta * P) *ᵥ u) := by
    rw [completionUnknownHalfContribution_eq_matrixPairing]
    change star u ⬝ᵥ (Delta *ᵥ (G *ᵥ v + P *ᵥ u)) = _
    rw [Matrix.mulVec_add, dotProduct_add,
      ← Matrix.mulVec_mulVec, ← Matrix.mulVec_mulVec]
  have hhalfStar : star (completionUnknownHalfContribution G lambda d u) =
      star v ⬝ᵥ ((G * Deltaᴴ) *ᵥ u) +
        star u ⬝ᵥ ((P * Deltaᴴ) *ᵥ u) := by
    rw [hhalf, star_add,
      conj_star_dotProduct_mulVec, conj_star_dotProduct_mulVec]
    simp only [Matrix.conjTranspose_mul, hG.eq, hP.eq]
  simp only [Fintype.sum_option, completionSampleVector, completionSamplePoint]
  rw [completionCorrectionKernel_zero_zero (G := G) d hd0]
  simp only [Matrix.zero_mulVec, dotProduct_zero, zero_add]
  simp_rw [completionCorrectionKernel_selected_zero_sample hG lambda d hd0]
  simp_rw [completionCorrectionKernel_selected_sample_zero lambda d hd0]
  simp_rw [completionCorrectionKernel_selected_sample_sample hG lambda d]
  rw [sum_sparse_right_pairing, Finset.sum_add_distrib,
    sum_sparse_left_pairing, sum_sparse_sparse_quadratic]
  change star v ⬝ᵥ ((G * Deltaᴴ) *ᵥ u) +
      (star u ⬝ᵥ ((Delta * G) *ᵥ v) +
        star u ⬝ᵥ ((Delta * P + P * Deltaᴴ) *ᵥ u)) = _
  rw [Matrix.add_mulVec, dotProduct_add]
  calc
    _ = (star u ⬝ᵥ ((Delta * G) *ᵥ v) +
          star u ⬝ᵥ ((Delta * P) *ᵥ u)) +
        (star v ⬝ᵥ ((G * Deltaᴴ) *ᵥ u) +
          star u ⬝ᵥ ((P * Deltaᴴ) *ᵥ u)) := by abel
    _ = completionUnknownHalfContribution G lambda d u +
        conj (completionUnknownHalfContribution G lambda d u) := by
      rw [← hhalf, ← hhalfStar]
      simp only [starRingEnd_apply]

/-- The compensating vector kills the residual coordinatewise, hence both the
unknown half-contribution and its conjugate vanish. -/
theorem exercise_05_solution
    [DecidableEq n]
    (G : SquareMatrix n) (hGunit : IsUnit G) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (u : n → ℂ) :
    G *ᵥ completionV G (completionP G lambda) u + completionP G lambda *ᵥ u = 0 ∧
      completionUnknownHalfContribution G lambda d u = 0 ∧
      completionUnknownHalfContribution G lambda d u +
        conj (completionUnknownHalfContribution G lambda d u) = 0 := by
  have hv := completion_mulVec_add_eq_zero G (completionP G lambda) u hGunit
  have hhalf := completionUnknownHalfContribution_eq_zero G lambda d u hGunit
  refine ⟨hv, hhalf, ?_⟩
  rw [hhalf]
  simp

/-- Universal nonnegativity of a Hermitian quadratic form is exactly the
positive-semidefinite criterion used after cancellation. -/
theorem exercise_06_solution
    [DecidableEq n]
    (Y : SquareMatrix n) (hY : Y.IsHermitian)
    (hquadratic : ∀ u : n → ℂ, 0 ≤ star u ⬝ᵥ (Y *ᵥ u)) : Y.PosSemidef := by
  exact Matrix.PosSemidef.of_dotProduct_mulVec_nonneg hY hquadratic

end Exercises.Chapter31

end
end CrouzeixTextbook.Part06
