import Mathlib

namespace TrainingDynamics

namespace Quadratic

/-- A quadratic gradient-descent update scales displacement from its minimizer. -/
theorem gd_error (a η x xStar : ℝ)
    (update : ℝ → ℝ) (hupdate : ∀ y, update y = y - η * a * (y - xStar)) :
    update x - xStar = (1 - η * a) * (x - xStar) := by
  rw [hupdate]
  ring

/-- Squared error after one quadratic gradient-descent step factors exactly. -/
theorem gd_energy_step (a η e : ℝ) :
    ((1 - η * a) * e)^2 = (1 - η * a)^2 * e^2 := by
  ring

/-- A step size in the stable quadratic interval yields a strict error contraction. -/
theorem gd_step_factor_abs_lt_one {a η : ℝ} (h_step_positive : 0 < η * a)
    (h_step_below_two : η * a < 2) : |1 - η * a| < 1 := by
  apply abs_lt.2
  constructor <;> linarith

end Quadratic

end TrainingDynamics
