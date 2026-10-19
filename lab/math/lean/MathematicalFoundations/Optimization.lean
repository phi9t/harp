import Mathlib.Analysis.SpecificLimits.Normed

/-!
Scalar quadratic optimization and stationary-iteration identities.

Only scalar recurrences and convergence under an explicit contraction are included here.
-/

namespace MathematicalFoundations.Optimization

/-- The derivative expression for `a * (x - c)^2` in one real coordinate. -/
def scalarQuadraticGradient (a c x : ℝ) : ℝ :=
  2 * a * (x - c)

/-- One scalar gradient-descent update for that quadratic. -/
def scalarGradientDescentStep (α a c x : ℝ) : ℝ :=
  x - α * scalarQuadraticGradient a c x

/-- The scalar gradient-descent error obeys its exact linear recurrence. -/
theorem scalar_gradient_descent_error_recurrence (α a c x : ℝ) :
    scalarGradientDescentStep α a c x - c = (1 - 2 * α * a) * (x - c) := by
  simp [scalarGradientDescentStep, scalarQuadraticGradient]
  ring

/-- The explicit `k`-step scalar stationary iteration around its fixed point. -/
def scalarStationaryIteration (q c x : ℝ) (k : ℕ) : ℝ :=
  c + q ^ k * (x - c)

/-- The explicit stationary iteration satisfies the one-step recurrence. -/
theorem scalar_stationary_iteration_succ (q c x : ℝ) (k : ℕ) :
    scalarStationaryIteration q c x (k + 1) =
      c + q * (scalarStationaryIteration q c x k - c) := by
  simp [scalarStationaryIteration, pow_succ]
  ring

/-- An explicit scalar contraction converges to its fixed point. -/
theorem scalar_stationary_iteration_tendsto_fixed_point {q c x : ℝ} (hq : |q| < 1) :
    Filter.Tendsto (fun k : ℕ => scalarStationaryIteration q c x k) Filter.atTop (nhds c) := by
  have hpow : Filter.Tendsto (fun k : ℕ => q ^ k) Filter.atTop (nhds 0) :=
    tendsto_pow_atTop_nhds_zero_of_abs_lt_one hq
  simpa [scalarStationaryIteration] using
    (tendsto_const_nhds.add (hpow.mul tendsto_const_nhds))

end MathematicalFoundations.Optimization
