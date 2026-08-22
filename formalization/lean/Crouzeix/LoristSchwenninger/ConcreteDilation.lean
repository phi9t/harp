import Crouzeix.LoristSchwenninger.CompanionAlgebra
import Crouzeix.LoristSchwenninger.CompressionMoments
import Crouzeix.LoristSchwenninger.PerturbationLemma
import Crouzeix.LoristSchwenninger.PolynomialPowerCauchy

/-!
The concrete Lorist--Schwenninger dilation associated to a polynomial on a
parametrized convex boundary.
-/

noncomputable section

open MeasureTheory
open scoped BoundedContinuousFunction Matrix Matrix.Norms.L2Operator

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {i n : Type*} [TopologicalSpace i] [CompactSpace i]
  [MeasurableSpace i] [BorelSpace i] [OpensMeasurableSpace i]
  [SecondCountableTopologyEither i ℂ]
  [Fintype n] [DecidableEq n] [Nonempty n]
  {mu : Measure i} [IsFiniteMeasure mu] {Omega : Set ℂ}

/-- The boundary multiplier dilation and its commuting companion
perturbations for a polynomial bounded by one on the closed domain. -/
def dilationDataOfParametricPolynomial
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (q : Polynomial ℂ)
    (hq : ∀ z ∈ closure Omega, ‖Polynomial.eval z q‖ ≤ 1)
    (hCauchy : HasParametricPolynomialCauchyFormula Gamma mu B) :
    DilationData (E := EuclideanVector n)
      (K := i →₂[mu] EuclideanVector n) := by
  let D := parametricPositiveBoundaryDensity Gamma B hWB hCauchy
  let h := parametricPolynomialBoundaryFunction Gamma q
  let P := polynomialEval q B
  let V := (boundaryEmbedding D).toContinuousLinearMap
  let Q := bcfMulL (mu := mu) (n := n) h
  let companion : ℕ → EuclideanVector n →L[ℂ] EuclideanVector n :=
    fun k ↦ euclideanOperator
      (parametricBoundaryCompanion Gamma mu B (h ^ k))
  let bound := ∫ x, ‖parametricBoundaryFirstPart Gamma B x‖ ∂mu
  refine
    { T := euclideanOperator P
      V := V
      Q := Q
      V_isometry := ?_
      Q_norm_le_one := ?_
      perturbation := companion
      perturbation_eq := ?_
      bound := bound
      bound_nonneg := ?_
      perturbation_norm_le := ?_
      commutes_with_target := ?_ }
  · exact (boundaryEmbedding D).isometry
  · exact bcfMulL_norm_le_one h
      (parametricPolynomialBoundaryFunction_norm_le_one Gamma q hq)
  · intro k
    have hcompression :=
      boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding
        D h k
    have hcompressionAdjoint :
        ContinuousLinearMap.adjoint
            ((ContinuousLinearMap.adjoint V).comp ((Q ^ k).comp V)) =
          ContinuousLinearMap.adjoint
            (euclideanOperator (boundaryPhiCLM D (h ^ k))) :=
      congrArg ContinuousLinearMap.adjoint hcompression
    rw [ContinuousLinearMap.adjoint_comp, ContinuousLinearMap.adjoint_comp,
      ContinuousLinearMap.adjoint_adjoint] at hcompressionAdjoint
    have hadjointPow :
        ContinuousLinearMap.adjoint (Q ^ k) =
          (ContinuousLinearMap.adjoint Q) ^ k := by
      rw [← ContinuousLinearMap.star_eq_adjoint, star_pow,
        ContinuousLinearMap.star_eq_adjoint]
    rw [hadjointPow, ← euclideanOperator_conjTranspose] at hcompressionAdjoint
    have hcompressed :
        compressedAdjointPower V Q k =
          euclideanOperator (boundaryPhiCLM D (h ^ k))ᴴ := by
      apply ContinuousLinearMap.ext
      intro x
      have hx := congrArg
        (fun A : EuclideanVector n →L[ℂ] EuclideanVector n ↦ A x)
        hcompressionAdjoint
      simpa only [compressedAdjointPower, ContinuousLinearMap.comp_apply] using hx
    have hCauchyPower :
        ∫ x, (h ^ k) x • parametricBoundaryFirstPart Gamma B x ∂mu =
          P ^ k := by
      simpa only [h, P, BoundedContinuousFunction.pow_apply] using
        parametricPolynomialPowerCauchy Gamma q B hCauchy k
    have hdouble := two_smul_boundaryPhi_parametric_eq
      Gamma B hWB hCauchy (h ^ k) (P ^ k) hCauchyPower
    have hdoubleAdjoint := congrArg Matrix.conjTranspose hdouble
    have hdoubleOperator :
        (2 : ℂ) • euclideanOperator (boundaryPhiCLM D (h ^ k))ᴴ =
          (euclideanOperator Pᴴ) ^ k + companion k := by
      have hmatrix :
          (2 : ℂ) • (boundaryPhiCLM D (h ^ k))ᴴ =
            (Pᴴ) ^ k +
              parametricBoundaryCompanion Gamma mu B (h ^ k) := by
        simpa only [D, Matrix.conjTranspose_smul, Matrix.conjTranspose_add,
          Matrix.conjTranspose_pow, Matrix.conjTranspose_conjTranspose,
          map_ofNat, star_ofNat] using hdoubleAdjoint
      simpa only [companion, map_smul, map_add, map_pow] using
        congrArg (fun A : SquareMatrix n ↦ euclideanOperator A) hmatrix
    have htargetAdjoint :
        ContinuousLinearMap.adjoint (euclideanOperator P) =
          euclideanOperator Pᴴ :=
      (euclideanOperator_conjTranspose P).symm
    change companion k =
      doubledCompressedAdjointPower V Q k -
        (ContinuousLinearMap.adjoint (euclideanOperator P)) ^ k
    rw [doubledCompressedAdjointPower, hcompressed, htargetAdjoint,
      hdoubleOperator]
    abel
  · exact integral_nonneg fun _ ↦ norm_nonneg _
  · intro k
    change ‖parametricBoundaryCompanion Gamma mu B (h ^ k)‖ ≤ bound
    calc
      ‖parametricBoundaryCompanion Gamma mu B (h ^ k)‖
          ≤ ‖h ^ k‖ * bound :=
        parametricBoundaryCompanion_norm_le Gamma B hWB (h ^ k)
      _ ≤ 1 * bound := mul_le_mul_of_nonneg_right
        (parametricPolynomialBoundaryFunction_pow_norm_le_one Gamma q hq k)
        (integral_nonneg fun _ ↦ norm_nonneg _)
      _ = bound := one_mul _
  · intro k
    have hcommute :=
      parametricBoundaryCompanion_commute_of_mem_generatedAlgebra
        (mu := mu) Gamma B P hWB (h ^ k)
        (polynomialEval_mem_generatedAlgebra (p := q) (A := B))
    rw [commute_iff_eq] at hcommute ⊢
    simpa only [companion, P, map_mul] using
      congrArg (fun A : SquareMatrix n ↦ euclideanOperator A) hcommute

/-- The Lorist--Schwenninger dilation gives the sharp factor-two bound for
the polynomial evaluation operator. -/
theorem norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (q : Polynomial ℂ)
    (hq : ∀ z ∈ closure Omega, ‖Polynomial.eval z q‖ ≤ 1)
    (hCauchy : HasParametricPolynomialCauchyFormula Gamma mu B) :
    ‖euclideanOperator (polynomialEval q B)‖ ≤ 2 := by
  have h :=
    (dilationDataOfParametricPolynomial Gamma B hWB q hq hCauchy).norm_target_le_two
  change ‖euclideanOperator (polynomialEval q B)‖ ≤ 2 at h
  exact h

end LoristSchwenninger
end CrouzeixConjecture
