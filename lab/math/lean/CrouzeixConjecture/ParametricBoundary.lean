module

public import CrouzeixConjecture.DoubleLayerPositiveMap
public import CrouzeixConjecture.HolomorphicMatrixAlgebra
public import Mathlib.Topology.Algebra.Polynomial
public import Mathlib.Topology.Instances.Matrix

@[expose] public section

noncomputable section

open MeasureTheory
open scoped BoundedContinuousFunction ComplexConjugate ComplexOrder Matrix
  Matrix.Norms.L2Operator

namespace CrouzeixConjecture

variable {i n : Type*} [TopologicalSpace i] [CompactSpace i]
  [MeasurableSpace i] [OpensMeasurableSpace i]
  [Fintype n] [DecidableEq n] [Nonempty n]

/-- A compact parametrized boundary carrying exactly the geometric data used by the
double-layer construction.  The measure on the parameter space is ordinary parameter
measure; `speed` is the arclength Jacobian.  No Cauchy formula is included in this structure. -/
structure ParametricConvexBoundary (Omega : Set ℂ) where
  point : C(i, ℂ)
  normal : C(i, ℂ)
  speed : C(i, ℝ)
  speed_nonneg : ∀ x, 0 ≤ speed x
  supported : ∀ x, OutwardBoundarySupport Omega (point x) (normal x)

variable {Omega : Set ℂ}

/-- The continuous resolvent along a parametrized supported boundary. -/
def parametricBoundaryResolvent
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (x : i) : SquareMatrix n :=
  doubleLayerResolvent B (Gamma.point x)

omit [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Nonempty n] in
/-- Resolvent inversion is continuous along the compact boundary because every boundary
point lies outside the numerical range, hence outside the spectrum. -/
theorem continuous_parametricBoundaryResolvent
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega) :
    Continuous (parametricBoundaryResolvent Gamma B) := by
  have hmatrix : Continuous
      (fun x : i ↦ Gamma.point x • (1 : SquareMatrix n) - B) := by
    fun_prop
  apply continuous_iff_continuousAt.2
  intro x
  have hunit := scalar_sub_matrix_isUnit_of_outwardBoundarySupport
    (Gamma.supported x) B hWB
  have hdet : IsUnit
      (Gamma.point x • (1 : SquareMatrix n) - B).det :=
    (Gamma.point x • (1 : SquareMatrix n) - B).isUnit_iff_isUnit_det.mp hunit
  have hinverse : ContinuousAt Ring.inverse
      (Gamma.point x • (1 : SquareMatrix n) - B).det := by
    simpa only [Ring.inverse_eq_inv'] using continuousAt_inv₀ hdet.ne_zero
  have hinverseMatrix : ContinuousAt (Inv.inv : SquareMatrix n → SquareMatrix n)
      (Gamma.point x • (1 : SquareMatrix n) - B) :=
    continuousAt_matrix_inv _ hinverse
  change ContinuousAt
    (fun y : i ↦ (Gamma.point y • (1 : SquareMatrix n) - B)⁻¹) x
  exact hinverseMatrix.comp'
    (f := fun y : i ↦ Gamma.point y • (1 : SquareMatrix n) - B)
    hmatrix.continuousAt

/-- The normalized analytic half of the pulled-back double-layer density.  The factor
`speed` converts parameter measure to arclength and `1/(2π)` is the manuscript's
normalization. -/
def parametricBoundaryFirstPart
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (x : i) : SquareMatrix n :=
  ((2 * Real.pi : ℝ)⁻¹ * Gamma.speed x) •
    (Gamma.normal x • parametricBoundaryResolvent Gamma B x)

/-- The pulled-back density is the analytic half plus its pointwise adjoint. -/
def parametricDoubleLayerDensity
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (x : i) : SquareMatrix n :=
  parametricBoundaryFirstPart Gamma B x +
    (parametricBoundaryFirstPart Gamma B x)ᴴ

omit [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Nonempty n] in
theorem continuous_parametricBoundaryFirstPart
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega) :
    Continuous (parametricBoundaryFirstPart Gamma B) := by
  unfold parametricBoundaryFirstPart
  have hres := continuous_parametricBoundaryResolvent Gamma B hWB
  fun_prop

omit [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Nonempty n] in
theorem continuous_parametricDoubleLayerDensity
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega) :
    Continuous (parametricDoubleLayerDensity Gamma B) := by
  unfold parametricDoubleLayerDensity
  exact (continuous_parametricBoundaryFirstPart Gamma B hWB).add
    (continuous_parametricBoundaryFirstPart Gamma B hWB).matrix_conjTranspose

omit [Nonempty n] in
/-- A continuous matrix-valued function on the compact parameter space is Bochner integrable
for every finite parameter measure. -/
theorem integrable_of_continuous_compact [IsFiniteMeasure (mu : Measure i)]
    {f : i → SquareMatrix n} (hf : Continuous f) : Integrable f mu := by
  let fb : i →ᵇ SquareMatrix n :=
    BoundedContinuousFunction.mkOfCompact ⟨f, hf⟩
  apply Integrable.of_bound fb.continuous.aestronglyMeasurable ‖fb‖
  exact Filter.Eventually.of_forall fun x ↦ fb.norm_coe_le_norm x

omit [Nonempty n] in
theorem integrable_parametricBoundaryFirstPart
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega) :
    Integrable (parametricBoundaryFirstPart Gamma B) mu :=
  integrable_of_continuous_compact
    (continuous_parametricBoundaryFirstPart Gamma B hWB)

omit [Nonempty n] in
/-- Multiplication by a bounded continuous scalar boundary function preserves integrability of
the analytic half of the double-layer density. -/
theorem integrable_parametricBoundaryBCFFirstPart
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (h : i →ᵇ ℂ) :
    Integrable (fun x ↦ h x • parametricBoundaryFirstPart Gamma B x) mu := by
  apply integrable_of_continuous_compact
  exact h.continuous.smul (continuous_parametricBoundaryFirstPart Gamma B hWB)

/-- Integration against the analytic half of the boundary density, as a linear map on continuous
boundary functions. -/
def parametricBoundaryFirstPartIntegralLinear
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega) :
    (i →ᵇ ℂ) →ₗ[ℂ] SquareMatrix n where
  toFun h := ∫ x, h x • parametricBoundaryFirstPart Gamma B x ∂mu
  map_add' h g := by
    simp only [BoundedContinuousFunction.add_apply, add_smul,
      integral_add (integrable_parametricBoundaryBCFFirstPart Gamma B hWB h)
        (integrable_parametricBoundaryBCFFirstPart Gamma B hWB g)]
  map_smul' c h := by
    simp only [BoundedContinuousFunction.smul_apply]
    have hfun :
        (fun x ↦ (c • h x) • parametricBoundaryFirstPart Gamma B x) =
          fun x ↦ c • (h x • parametricBoundaryFirstPart Gamma B x) := by
      funext x
      simp [smul_smul]
    rw [hfun, integral_smul]
    rfl

omit [Nonempty n] in
theorem parametricBoundaryFirstPartIntegral_norm_le
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (h : i →ᵇ ℂ) :
    ‖∫ x, h x • parametricBoundaryFirstPart Gamma B x ∂mu‖ ≤
      (∫ x, ‖parametricBoundaryFirstPart Gamma B x‖ ∂mu) * ‖h‖ := by
  have hpoint : ∀ x,
      ‖h x • parametricBoundaryFirstPart Gamma B x‖ ≤
        ‖h‖ * ‖parametricBoundaryFirstPart Gamma B x‖ := by
    intro x
    rw [norm_smul]
    exact mul_le_mul_of_nonneg_right (h.norm_coe_le_norm x) (norm_nonneg _)
  calc
    ‖∫ x, h x • parametricBoundaryFirstPart Gamma B x ∂mu‖ ≤
        ∫ x, ‖h x • parametricBoundaryFirstPart Gamma B x‖ ∂mu :=
      norm_integral_le_integral_norm _
    _ ≤ ∫ x, ‖h‖ * ‖parametricBoundaryFirstPart Gamma B x‖ ∂mu := by
      exact integral_mono_ae
        (integrable_parametricBoundaryBCFFirstPart Gamma B hWB h).norm
        ((integrable_parametricBoundaryFirstPart Gamma B hWB).norm.const_mul ‖h‖)
        (Filter.Eventually.of_forall hpoint)
    _ = (∫ x, ‖parametricBoundaryFirstPart Gamma B x‖ ∂mu) * ‖h‖ := by
      rw [integral_const_mul]
      ring

/-- The boundary integral is continuous in the uniform norm. -/
def parametricBoundaryFirstPartIntegralCLM
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega) :
    (i →ᵇ ℂ) →L[ℂ] SquareMatrix n :=
  (parametricBoundaryFirstPartIntegralLinear Gamma B hWB).mkContinuous
    (∫ x, ‖parametricBoundaryFirstPart Gamma B x‖ ∂mu)
    (parametricBoundaryFirstPartIntegral_norm_le Gamma B hWB)

omit [Nonempty n] in
@[simp]
theorem parametricBoundaryFirstPartIntegralCLM_apply
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (h : i →ᵇ ℂ) :
    parametricBoundaryFirstPartIntegralCLM (mu := mu) Gamma B hWB h =
      ∫ x, h x • parametricBoundaryFirstPart Gamma B x ∂mu := rfl

omit [Nonempty n] in
theorem integrable_parametricDoubleLayerDensity
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega) :
    Integrable (parametricDoubleLayerDensity Gamma B) mu :=
  integrable_of_continuous_compact
    (continuous_parametricDoubleLayerDensity Gamma B hWB)

omit [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Nonempty n] in
/-- Pointwise positivity of the pulled-back density follows from the supporting-line
inequality and the nonnegative arclength speed. -/
theorem parametricDoubleLayerDensity_posSemidef
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega) (x : i) :
    (parametricDoubleLayerDensity Gamma B x).PosSemidef := by
  have hbase := normalizedDoubleLayerResolvent_density_posSemidef
    (Gamma.supported x) B hWB
  have hscaled := hbase.smul (Gamma.speed_nonneg x)
  have heq :
      parametricDoubleLayerDensity Gamma B x =
        Gamma.speed x • (((2 * Real.pi : ℝ)⁻¹ •
          doubleLayerDensity
            (parametricBoundaryResolvent Gamma B x) (Gamma.normal x))) := by
    ext a b
    simp [parametricDoubleLayerDensity, parametricBoundaryFirstPart,
      parametricBoundaryResolvent, doubleLayerDensity, smul_add,
      Complex.real_smul]
    ring
  rw [heq]
  exact hscaled

/-- The exact remaining analytic statement for a fixed parametrized contour: polynomial
Cauchy evaluation for the normalized analytic half of the density.  Unlike the previous
provider interface, this proposition exposes the contour integral that still has to be proved
from the radial convex-boundary construction. -/
def HasParametricPolynomialCauchyFormula
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (mu : Measure i) (B : SquareMatrix n) : Prop :=
  ∀ q : Polynomial ℂ,
    ∫ x, Polynomial.eval (Gamma.point x) q •
      parametricBoundaryFirstPart Gamma B x ∂mu = polynomialEval q B

/-- A polynomial restricted to the compact parametrized boundary, bundled as a bounded
continuous function. -/
def parametricPolynomialBoundaryFunction
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (q : Polynomial ℂ) : i →ᵇ ℂ :=
  BoundedContinuousFunction.mkOfCompact
    ⟨fun x ↦ Polynomial.eval (Gamma.point x) q,
      q.continuous.comp Gamma.point.continuous⟩

omit [MeasurableSpace i] [OpensMeasurableSpace i] in
@[simp]
theorem parametricPolynomialBoundaryFunction_apply
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (q : Polynomial ℂ) (x : i) :
    parametricPolynomialBoundaryFunction Gamma q x =
      Polynomial.eval (Gamma.point x) q := rfl

omit [Nonempty n] in
theorem integrable_parametricPolynomialFirstPart
    [IsFiniteMeasure (mu : Measure i)]
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (q : Polynomial ℂ) :
    Integrable (fun x ↦ Polynomial.eval (Gamma.point x) q •
      parametricBoundaryFirstPart Gamma B x) mu := by
  apply integrable_of_continuous_compact
  exact (q.continuous.comp Gamma.point.continuous).smul
    (continuous_parametricBoundaryFirstPart Gamma B hWB)

end CrouzeixConjecture
