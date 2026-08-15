import Mathlib

namespace TrainingDynamics

namespace Momentum

/-- One scalar heavy-ball update about the optimum `xStar`.

This file proves algebraic recurrence identities only; it makes no general
stability or convergence claim.
-/
def heavyBall (a η β x previous xStar : ℝ) : ℝ :=
  x - η * a * (x - xStar) + β * (x - previous)

theorem heavyBall_error (a η β x previous xStar : ℝ) :
    heavyBall a η β x previous xStar - xStar =
      (1 - η * a + β) * (x - xStar) - β * (previous - xStar) := by
  simp only [heavyBall]
  ring

theorem heavyBall_zero_momentum (a η x previous xStar : ℝ) :
    heavyBall a η 0 x previous xStar = x - η * a * (x - xStar) := by
  simp [heavyBall]

end Momentum

end TrainingDynamics
