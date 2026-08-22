import CrouzeixConjecture.Euclidean
import Mathlib.MeasureTheory.Function.Holder
import Mathlib.MeasureTheory.Function.L2Space
import Mathlib.MeasureTheory.Function.LpSpace.ContinuousFunctions

/-!
The pointwise action of a bounded continuous scalar function on the
Lorist--Schwenninger boundary `L²` space.
-/

noncomputable section

open MeasureTheory
open scoped BoundedContinuousFunction

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {i n : Type*} [MeasurableSpace i] [TopologicalSpace i] [BorelSpace i]
  [SecondCountableTopologyEither i ℂ] [Fintype n]
variable {mu : Measure i}

/-- Pointwise multiplication by a bounded continuous scalar preserves boundary `L²`. -/
theorem bcfMulMemLp (h : i →ᵇ ℂ) (f : i →₂[mu] EuclideanVector n) :
    MemLp (fun z ↦ h z • f z) 2 mu := by
  apply (Lp.memLp f).of_le_mul
  · exact h.continuous.aestronglyMeasurable.smul (Lp.aestronglyMeasurable f)
  · filter_upwards with z
    rw [norm_smul]
    exact mul_le_mul_of_nonneg_right (h.norm_coe_le_norm z) (norm_nonneg (f z))

/-- Pointwise multiplication by a bounded continuous scalar on boundary `L²`. -/
def bcfMul (h : i →ᵇ ℂ) (f : i →₂[mu] EuclideanVector n) :
    i →₂[mu] EuclideanVector n :=
  (bcfMulMemLp h f).toLp (fun z ↦ h z • f z)

/-- The multiplier acts by pointwise scalar multiplication almost everywhere. -/
theorem bcfMul_apply_ae (h : i →ᵇ ℂ) (f : i →₂[mu] EuclideanVector n) :
    bcfMul h f =ᵐ[mu] fun z ↦ h z • f z := by
  exact MemLp.coeFn_toLp (bcfMulMemLp h f)

/-- Pointwise multiplication has the expected sup-norm bound. -/
theorem bcfMul_norm_le (h : i →ᵇ ℂ) (f : i →₂[mu] EuclideanVector n) :
    ‖bcfMul h f‖ ≤ ‖h‖ * ‖f‖ := by
  apply Lp.norm_le_mul_norm_of_ae_le_mul
  filter_upwards [bcfMul_apply_ae h f] with z hz
  rw [hz, norm_smul]
  exact mul_le_mul_of_nonneg_right (h.norm_coe_le_norm z) (norm_nonneg (f z))

/-- Pointwise multiplication by a fixed bounded continuous scalar, as a
complex linear map on boundary `L²`. -/
def bcfMulLinear (h : i →ᵇ ℂ) :
    (i →₂[mu] EuclideanVector n) →ₗ[ℂ] (i →₂[mu] EuclideanVector n) where
  toFun := bcfMul h
  map_add' f g := by
    apply Lp.ext
    filter_upwards [bcfMul_apply_ae h (f + g), bcfMul_apply_ae h f,
      bcfMul_apply_ae h g, Lp.coeFn_add f g,
      Lp.coeFn_add (bcfMul h f) (bcfMul h g)] with z hfg hf hg hsum hout
    rw [hfg, hout, hsum]
    simp only [Pi.add_apply, hf, hg, smul_add]
  map_smul' c f := by
    apply Lp.ext
    filter_upwards [bcfMul_apply_ae h (c • f), bcfMul_apply_ae h f,
      Lp.coeFn_smul c f, Lp.coeFn_smul c (bcfMul h f)]
      with z hcf hf hin hout
    rw [hcf, hin]
    simp only [RingHom.id_apply]
    rw [hout]
    simp only [Pi.smul_apply, hf, smul_smul, mul_comm]

/-- Pointwise multiplication as a complex continuous linear endomorphism. -/
def bcfMulL (h : i →ᵇ ℂ) :
    (i →₂[mu] EuclideanVector n) →L[ℂ] (i →₂[mu] EuclideanVector n) :=
  (bcfMulLinear h).mkContinuous ‖h‖ (bcfMul_norm_le h)

/-- The bundled multiplier acts pointwise almost everywhere. -/
theorem bcfMulL_apply_ae (h : i →ᵇ ℂ) (f : i →₂[mu] EuclideanVector n) :
    bcfMulL h f =ᵐ[mu] fun z ↦ h z • f z := by
  exact bcfMul_apply_ae h f

/-- The operator norm of the multiplier is bounded by the scalar sup norm. -/
theorem bcfMulL_norm_le (h : i →ᵇ ℂ) :
    ‖bcfMulL (mu := mu) (n := n) h‖ ≤ ‖h‖ := by
  exact LinearMap.mkContinuous_norm_le _ (norm_nonneg h) (bcfMul_norm_le h)

/-- Multiplication by the constant one function is the identity. -/
@[simp]
theorem bcfMulL_one :
    bcfMulL (mu := mu) (n := n) (1 : i →ᵇ ℂ) = 1 := by
  apply ContinuousLinearMap.ext
  intro f
  apply Lp.ext
  filter_upwards [bcfMulL_apply_ae (1 : i →ᵇ ℂ) f] with z hz
  simpa using hz

/-- Products of scalar functions act by composition of their multipliers. -/
theorem bcfMulL_mul (h g : i →ᵇ ℂ) :
    bcfMulL (mu := mu) (n := n) (h * g) =
      bcfMulL h * bcfMulL g := by
  apply ContinuousLinearMap.ext
  intro f
  apply Lp.ext
  filter_upwards [bcfMulL_apply_ae (h * g) f,
      bcfMulL_apply_ae h (bcfMulL g f), bcfMulL_apply_ae g f]
    with z hl hr hg
  rw [hl, mul_apply_eq_comp, hr, hg]
  simp only [BoundedContinuousFunction.mul_apply, mul_smul]

/-- Powers of a scalar function act by powers of its multiplier. -/
theorem bcfMulL_pow (h : i →ᵇ ℂ) (k : ℕ) :
    bcfMulL (mu := mu) (n := n) (h ^ k) = (bcfMulL h) ^ k := by
  induction k with
  | zero => simp
  | succ k ih =>
      rw [pow_succ, bcfMulL_mul, ih, pow_succ]

/-- A bounded continuous scalar of sup norm at most one induces a contraction. -/
theorem bcfMulL_norm_le_one (h : i →ᵇ ℂ) (hh : ‖h‖ ≤ 1) :
    ‖bcfMulL (mu := mu) (n := n) h‖ ≤ 1 := by
  exact (bcfMulL_norm_le h).trans hh

end LoristSchwenninger
end CrouzeixConjecture
