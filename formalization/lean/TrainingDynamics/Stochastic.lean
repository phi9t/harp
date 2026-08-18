import Mathlib

namespace TrainingDynamics

namespace Stochastic

/--
This module formalizes only a finite-support expectation identity for a
two-outcome symmetric noise model. It does not establish general SGD
convergence.
-/
noncomputable def meanTwo (left right : ℝ) : ℝ := (left + right) / 2

theorem symmetric_noise_is_unbiased (gradient δ : ℝ) :
    meanTwo (gradient + δ) (gradient - δ) = gradient := by
  unfold meanTwo
  ring_nf

end Stochastic

end TrainingDynamics
