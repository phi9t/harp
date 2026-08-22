import Crouzeix.LoristSchwenninger.BoundarySquareRoot
import Mathlib.Analysis.SpecialFunctions.ContinuousFunctionalCalculus.Rpow.Isometric
import Mathlib.MeasureTheory.Function.L2Space

/-!
The Lorist--Schwenninger boundary embedding into the concrete vector-valued
`L²` space.
-/

noncomputable section

open MeasureTheory
open scoped ComplexOrder MatrixOrder Matrix.Norms.L2Operator

namespace CrouzeixConjecture
namespace LoristSchwenninger

set_option maxHeartbeats 800000

variable {i n : Type*} [MeasurableSpace i]
  [Fintype n] [DecidableEq n]
variable {mu : Measure i}

/-- The canonical boundary square root is almost everywhere strongly
measurable, despite the density being nonnegative only almost everywhere. -/
theorem boundarySquareRoot_aestronglyMeasurable
    (D : PositiveBoundaryDensity (n := n) mu) :
    AEStronglyMeasurable (boundarySquareRoot D) mu := by
  have hD : AEStronglyMeasurable D.density mu :=
    D.integrable_density.aestronglyMeasurable
  have hD_nonneg : ∀ᵐ z ∂mu, 0 ≤ D.density z :=
    D.posSemidef_ae.mono fun _ hz ↦ hz.nonneg
  letI : MeasurableSpace (SquareMatrix n) := borel (SquareMatrix n)
  haveI : BorelSpace (SquareMatrix n) := ⟨rfl⟩
  obtain ⟨g, hg_meas, hg_nonneg, hDg⟩ :=
    hD.aemeasurable.exists_measurable_nonneg hD_nonneg
  let gpos : i → {A : SquareMatrix n | 0 ≤ A} :=
    fun z ↦ ⟨g z, hg_nonneg z⟩
  have hgpos : StronglyMeasurable gpos :=
    hg_meas.subtype_mk.stronglyMeasurable
  have hsqrt : StronglyMeasurable (fun z ↦ CFC.sqrt (g z)) := by
    simpa [gpos] using
      CFC.continuousOn_sqrt.restrict.comp_stronglyMeasurable hgpos
  apply hsqrt.aestronglyMeasurable.congr
  filter_upwards [hDg] with z hz
  simp [boundarySquareRoot, hz]

/-- The normalized square-root boundary vector field associated to `x`. -/
def boundaryEmbeddingField
    (D : PositiveBoundaryDensity (n := n) mu)
    (x : EuclideanVector n) (z : i) : EuclideanVector n :=
  ((Real.sqrt 2 : ℂ)⁻¹) • euclideanOperator (boundarySquareRoot D z) x

/-- Pointwise, the square norm of the boundary field is the normalized
quadratic form of the density, almost everywhere. -/
theorem boundaryEmbeddingField_sq_norm_ae
    (D : PositiveBoundaryDensity (n := n) mu) (x : EuclideanVector n) :
    ∀ᵐ z ∂mu,
      ‖boundaryEmbeddingField D x z‖ ^ 2 =
        (2 : ℝ)⁻¹ * Complex.re
          (euclideanQuadraticFormCLM x (D.density z)) := by
  filter_upwards [boundarySquareRoot_mul_self_ae D] with z hsq
  rw [boundaryEmbeddingField, norm_smul, mul_pow, norm_inv, Complex.norm_real]
  norm_num [Real.sq_sqrt]
  rw [norm_sq_eq_re_inner (𝕜 := ℂ)]
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
  change Complex.re (euclideanQuadraticFormCLM x (D.density z)) =
    Complex.re (euclideanQuadraticFormCLM x (D.density z))
  rfl

/-- The normalized density quadratic form integrates to the Euclidean square
norm, by the boundary-mass identity. -/
theorem integral_normalized_boundaryQuadraticForm
    (D : PositiveBoundaryDensity (n := n) mu) (x : EuclideanVector n) :
    ∫ z, (2 : ℝ)⁻¹ * Complex.re
      (euclideanQuadraticFormCLM x (D.density z)) ∂mu = ‖x‖ ^ 2 := by
  have hq : Integrable
      (fun z => euclideanQuadraticFormCLM x (D.density z)) mu :=
    (euclideanQuadraticFormCLM x).integrable_comp D.integrable_density
  have hre :
      ∫ z, Complex.re (euclideanQuadraticFormCLM x (D.density z)) ∂mu =
        Complex.re (∫ z, euclideanQuadraticFormCLM x (D.density z) ∂mu) :=
    integral_re hq
  rw [integral_const_mul, hre,
    (euclideanQuadraticFormCLM x).integral_comp_comm D.integrable_density,
    D.mass_eq_two_one, map_smul]
  simp [euclideanQuadraticFormCLM_apply, inner_self_eq_norm_sq_to_K]
  norm_cast

/-- Each normalized boundary vector field belongs to the concrete vector-valued
`L²` space. -/
theorem boundaryEmbeddingField_memLp
    (D : PositiveBoundaryDensity (n := n) mu) (x : EuclideanVector n) :
    MemLp (boundaryEmbeddingField D x) 2 mu := by
  have hmeas : AEStronglyMeasurable (boundaryEmbeddingField D x) mu := by
    change AEStronglyMeasurable
      (((Real.sqrt 2 : ℂ)⁻¹) •
        fun z ↦ euclideanOperator (boundarySquareRoot D z) x) mu
    apply AEStronglyMeasurable.const_smul
    exact (euclideanOperatorCLM.continuous.comp_aestronglyMeasurable
      (boundarySquareRoot_aestronglyMeasurable D)).apply_continuousLinearMap x
  rw [memLp_two_iff_integrable_sq_norm hmeas]
  have hq : Integrable
      (fun z => euclideanQuadraticFormCLM x (D.density z)) mu :=
    (euclideanQuadraticFormCLM x).integrable_comp D.integrable_density
  exact (hq.re.const_mul (2 : ℝ)⁻¹).congr
    (Filter.EventuallyEq.symm (boundaryEmbeddingField_sq_norm_ae D x))

/-- The concrete `L²` representative of the boundary vector field. -/
def boundaryEmbeddingToLp
    (D : PositiveBoundaryDensity (n := n) mu) (x : EuclideanVector n) :
    i →₂[mu] EuclideanVector n :=
  (boundaryEmbeddingField_memLp D x).toLp (boundaryEmbeddingField D x)

/-- Passing the normalized boundary field to `L²` is complex-linear. -/
def boundaryEmbeddingLinear
    (D : PositiveBoundaryDensity (n := n) mu) :
    EuclideanVector n →ₗ[ℂ] (i →₂[mu] EuclideanVector n) where
  toFun := boundaryEmbeddingToLp D
  map_add' x y := by
    simp only [boundaryEmbeddingToLp]
    rw [← MemLp.toLp_add]
    apply MemLp.toLp_congr
    exact Filter.Eventually.of_forall fun z => by
      simp [boundaryEmbeddingField, map_add]
  map_smul' c x := by
    simp only [boundaryEmbeddingToLp]
    rw [← MemLp.toLp_const_smul]
    apply MemLp.toLp_congr
    exact Filter.Eventually.of_forall fun z => by
      simp only [boundaryEmbeddingField, Pi.smul_apply, map_smul, smul_smul, RingHom.id_apply]
      rw [mul_comm]

/-- The concrete `L²` representative has the same norm as its source vector. -/
theorem norm_boundaryEmbeddingToLp
    (D : PositiveBoundaryDensity (n := n) mu) (x : EuclideanVector n) :
    ‖boundaryEmbeddingToLp D x‖ = ‖x‖ := by
  apply (sq_eq_sq₀ (norm_nonneg _) (norm_nonneg _)).mp
  rw [norm_sq_eq_re_inner (𝕜 := ℂ), MeasureTheory.L2.inner_def]
  rw [← integral_re (MeasureTheory.L2.integrable_inner
    (boundaryEmbeddingToLp D x) (boundaryEmbeddingToLp D x))]
  rw [integral_congr_ae]
  · exact integral_normalized_boundaryQuadraticForm D x
  · filter_upwards
      [(boundaryEmbeddingField_memLp D x).coeFn_toLp,
        boundaryEmbeddingField_sq_norm_ae D x] with z hz hsqz
    change boundaryEmbeddingToLp D x z = boundaryEmbeddingField D x z at hz
    rw [hz, inner_self_eq_norm_sq]
    exact hsqz

/-- The normalized square-root boundary field gives an isometric embedding
into the concrete vector-valued `L²` space. -/
def boundaryEmbedding
    (D : PositiveBoundaryDensity (n := n) mu) :
    EuclideanVector n →ₗᵢ[ℂ] (i →₂[mu] EuclideanVector n) where
  toLinearMap := boundaryEmbeddingLinear D
  norm_map' := norm_boundaryEmbeddingToLp D

end LoristSchwenninger
end CrouzeixConjecture
