import Crouzeix.LoristSchwenninger.BoundaryEmbedding
import Crouzeix.LoristSchwenninger.BoundaryMultiplier

/-!
Compression identities for the Lorist--Schwenninger boundary embedding.
-/

noncomputable section

open MeasureTheory
open scoped BoundedContinuousFunction ComplexOrder MatrixOrder Matrix.Norms.L2Operator

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {i n : Type*} [MeasurableSpace i] [TopologicalSpace i] [BorelSpace i]
  [SecondCountableTopologyEither i ℂ] [Fintype n] [DecidableEq n]
variable {mu : Measure i}

theorem boundaryEmbedding_adjoint_comp_bcfMulL_comp_boundaryEmbedding
    (D : PositiveBoundaryDensity (n := n) mu) (h : i →ᵇ ℂ) :
    (ContinuousLinearMap.adjoint (boundaryEmbedding D).toContinuousLinearMap).comp
        ((bcfMulL (mu := mu) (n := n) h).comp
          (boundaryEmbedding D).toContinuousLinearMap) =
      euclideanOperator (boundaryPhiCLM D h) := by
  apply ContinuousLinearMap.ext
  intro x
  apply ext_inner_left ℂ
  intro y
  rw [ContinuousLinearMap.comp_apply, ContinuousLinearMap.adjoint_inner_right]
  rw [ContinuousLinearMap.comp_apply]
  rw [MeasureTheory.L2.inner_def]
  have hx :
      boundaryEmbedding D x =ᵐ[mu] boundaryEmbeddingField D x :=
    (boundaryEmbeddingField_memLp D x).coeFn_toLp
  have hy :
      boundaryEmbedding D y =ᵐ[mu] boundaryEmbeddingField D y :=
    (boundaryEmbeddingField_memLp D y).coeFn_toLp
  have hmul :
      bcfMulL h (boundaryEmbedding D x) =ᵐ[mu] fun z ↦ h z • boundaryEmbedding D x z :=
    bcfMulL_apply_ae h (boundaryEmbedding D x)
  have hcongr :
      (fun z ↦ inner ℂ (boundaryEmbedding D y z) (bcfMulL h (boundaryEmbedding D x) z)) =ᵐ[mu]
        (fun z ↦ (2 : ℂ)⁻¹ * inner ℂ y (euclideanOperator (h z • D.density z) x)) := by
    filter_upwards [hy, hmul, hx, boundarySquareRoot_mul_self_ae D] with z hyz hmz hxz hsq
    rw [hyz, hmz, hxz, boundaryEmbeddingField, boundaryEmbeddingField]
    have hsqrt :
        inner ℂ (euclideanOperator (boundarySquareRoot D z) y)
            (euclideanOperator (boundarySquareRoot D z) x) =
          inner ℂ y (euclideanOperator (D.density z) x) := by
      rw [← ContinuousLinearMap.adjoint_inner_right]
      rw [← euclideanOperator_conjTranspose,
        (boundarySquareRoot_posSemidef D z).isHermitian.eq]
      have happly :
          euclideanOperator (boundarySquareRoot D z)
              (euclideanOperator (boundarySquareRoot D z) x) =
            euclideanOperator (D.density z) x := by
        have := congrArg (fun A : SquareMatrix n ↦ euclideanOperator A x) hsq
        simpa only [map_mul, mul_apply_eq_comp] using this
      rw [happly]
    rw [inner_smul_left, inner_smul_right, inner_smul_right, hsqrt]
    rw [map_smul, smul_apply, inner_smul_right]
    have hsqrt_ne : (Real.sqrt 2 : ℂ) ≠ 0 := by
      exact_mod_cast Real.sqrt_ne_zero'.mpr two_pos
    have hstar :
        (starRingEnd ℂ) ((Real.sqrt 2 : ℂ)⁻¹) =
          ((Real.sqrt 2 : ℂ)⁻¹) := by
      rw [map_inv₀]
      norm_num
    have hhalf :
        ((Real.sqrt 2 : ℂ)⁻¹) * ((Real.sqrt 2 : ℂ)⁻¹) =
          (2 : ℂ)⁻¹ := by
      field_simp [hsqrt_ne]
      have hsqrt_sq : Real.sqrt 2 ^ 2 = (2 : ℝ) :=
        Real.sq_sqrt (by norm_num)
      exact_mod_cast hsqrt_sq.symm
    rw [hstar]
    calc
      ((Real.sqrt 2 : ℂ)⁻¹) *
          (h z * (((Real.sqrt 2 : ℂ)⁻¹) *
            inner ℂ y (euclideanOperator (D.density z) x))) =
          (((Real.sqrt 2 : ℂ)⁻¹) * ((Real.sqrt 2 : ℂ)⁻¹)) *
            (h z * inner ℂ y (euclideanOperator (D.density z) x)) := by
              ring
      _ = (2 : ℂ)⁻¹ *
          (h z * inner ℂ y (euclideanOperator (D.density z) x)) := by
            rw [hhalf]
  change
    ∫ a : i, inner ℂ (((boundaryEmbedding D).toContinuousLinearMap) y a)
        (((bcfMulL h) (((boundaryEmbedding D).toContinuousLinearMap) x)) a) ∂mu =
      inner ℂ y (euclideanOperator (boundaryPhiCLM D h) x)
  have hcongr' :
      (fun z ↦ inner ℂ
        (((boundaryEmbedding D).toContinuousLinearMap) y z)
        (bcfMulL h (((boundaryEmbedding D).toContinuousLinearMap) x) z)) =ᵐ[mu]
        (fun z ↦ (2 : ℂ)⁻¹ *
          inner ℂ y (euclideanOperator (h z • D.density z) x)) := by
    filter_upwards [hcongr] with z hz
    change inner ℂ (boundaryEmbedding D y z)
      (bcfMulL h (boundaryEmbedding D x) z) = _
    exact hz
  rw [integral_congr_ae hcongr']
  have h_int :
      Integrable (fun z ↦ h z • D.density z) mu :=
    D.integrable_smul h
  let L :
      SquareMatrix n →L[ℂ] EuclideanVector n :=
    (ContinuousLinearMap.apply ℂ (EuclideanVector n) x).comp euclideanOperatorCLM
  have h_int_vec :
      Integrable (fun z ↦ euclideanOperator (h z • D.density z) x) mu :=
    L.integrable_comp h_int
  have h_integral :
      ∫ z, euclideanOperator (h z • D.density z) x ∂mu =
        euclideanOperator (∫ z, h z • D.density z ∂mu) x := by
    simpa [L, euclideanOperatorCLM_apply] using L.integral_comp_comm h_int
  calc
    ∫ z, (2 : ℂ)⁻¹ * inner ℂ y (euclideanOperator (h z • D.density z) x) ∂mu
        = (2 : ℂ)⁻¹ * ∫ z, inner ℂ y (euclideanOperator (h z • D.density z) x) ∂mu := by
            rw [integral_const_mul]
    _ = (2 : ℂ)⁻¹ * inner ℂ y (∫ z, euclideanOperator (h z • D.density z) x ∂mu) := by
          rw [integral_inner h_int_vec y]
    _ = inner ℂ y ((2 : ℂ)⁻¹ • (euclideanOperator (∫ z, h z • D.density z ∂mu) x)) := by
          rw [h_integral, inner_smul_right]
    _ = inner ℂ y (euclideanOperator (boundaryPhiCLM D h) x) := by
          rw [boundaryPhiCLM_apply, boundaryPhi]
          simp only [map_smul, smul_apply]

theorem boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding
    (D : PositiveBoundaryDensity (n := n) mu) (h : i →ᵇ ℂ) (k : ℕ) :
    (ContinuousLinearMap.adjoint (boundaryEmbedding D).toContinuousLinearMap).comp
        (((bcfMulL (mu := mu) (n := n) h) ^ k).comp
          (boundaryEmbedding D).toContinuousLinearMap) =
      euclideanOperator (boundaryPhiCLM D (h ^ k)) := by
  rw [← bcfMulL_pow (mu := mu) (n := n) h k]
  exact boundaryEmbedding_adjoint_comp_bcfMulL_comp_boundaryEmbedding D (h ^ k)

end LoristSchwenninger
end CrouzeixConjecture
