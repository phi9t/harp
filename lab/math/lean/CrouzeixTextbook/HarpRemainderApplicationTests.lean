import Crouzeix.Harp.FiniteHorizonRemainderApplication

open CrouzeixConjecture.Harp

#check finite_horizon_displacement_envelope_nonneg
#check finite_horizon_scalar_lower_bound_with_remainder

noncomputable section
namespace CrouzeixTextbook.HarpRemainderApplicationTests

variable {E K : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace ℂ E] [CompleteSpace E] [Nontrivial E]
variable [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]

/-- This client has one horizon witness and no finite-dimensionality assumption. -/
theorem one_horizon_client {core : CommutingPerturbationData E} {N : ℕ}
    (data : FiniteHorizonDilationData core K N) {κ : ℝ} (hκ : 1 < κ)
    (hnorm : κ = ‖core.T‖) (x : E) (hx : ‖x‖ = 1)
    (hsingular : ContinuousLinearMap.adjoint core.T (core.T x) =
      ((κ ^ 2 : ℝ) : ℂ) • x) :
    let m := core.recurrenceScalar x
    let M := core.bound * (2 + core.bound)
    let C := 2 * κ ^ 2 - κ * m 1 - κ ^ 3
    let c := -C / (κ ^ 2 - κ)
    c / (κ - 1) - (κ⁻¹) ^ N * (M + |c| / (κ - 1)) ≤ m 1 := by
  exact finite_horizon_scalar_lower_bound_with_remainder data hκ hnorm x hx hsingular

omit [Nontrivial E] in
theorem envelope_client {core : CommutingPerturbationData E} {N : ℕ}
    (data : FiniteHorizonDilationData core K N) {κ : ℝ} {x : E}
    (hx : ‖x‖ = 1) (hnorm : κ = ‖core.T‖)
    (hsingular : ContinuousLinearMap.adjoint core.T (core.T x) =
      ((κ ^ 2 : ℝ) : ℂ) • x) :
    0 ≤ 2 * κ ^ 2 - κ * core.recurrenceScalar x 1 - κ ^ 3 := by
  exact finite_horizon_displacement_envelope_nonneg data hx hnorm hsingular

#print axioms finite_horizon_displacement_envelope_nonneg
#print axioms finite_horizon_scalar_lower_bound_with_remainder
#print axioms one_horizon_client
#print axioms envelope_client

end CrouzeixTextbook.HarpRemainderApplicationTests
