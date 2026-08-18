import MathematicalFoundations.Linear
import MathematicalFoundations.Orthogonality
import MathematicalFoundations.Probability
import MathematicalFoundations.BayesInformation
import MathematicalFoundations.LinearModels
import MathematicalFoundations.Optimization

/-!
Machine-readable inventory of the public theorem boundary for the Mathematical
Foundations companion. The `#check` commands bind each inventory name to a
declaration compiled by the root project.
-/

namespace MathematicalFoundations

def publicTheoremManifest : List String := [
  "MathematicalFoundations.Linear.linear_map_zero",
  "MathematicalFoundations.Linear.linear_map_add",
  "MathematicalFoundations.Linear.linear_map_smul",
  "MathematicalFoundations.Linear.linear_kernel_zero",
  "MathematicalFoundations.Linear.linear_kernel_add",
  "MathematicalFoundations.Linear.linear_kernel_smul",
  "MathematicalFoundations.Linear.linear_finite_coordinate_reconstruction",
  "MathematicalFoundations.Orthogonality.line_projection_residual_decomposition",
  "MathematicalFoundations.Orthogonality.line_projection_residual_inner_eq_zero",
  "MathematicalFoundations.Orthogonality.symmetric_positive_definite_quadratic_positive",
  "MathematicalFoundations.Probability.expectation_add",
  "MathematicalFoundations.Probability.expectation_smul",
  "MathematicalFoundations.Probability.expectation_sub",
  "MathematicalFoundations.Probability.expectation_const",
  "MathematicalFoundations.Probability.covariance_expand",
  "MathematicalFoundations.BayesInformation.eventProbability_and_comm",
  "MathematicalFoundations.BayesInformation.conditional_mul_evidence",
  "MathematicalFoundations.BayesInformation.bayes_rule",
  "MathematicalFoundations.LinearModels.normal_equation_residual_identity",
  "MathematicalFoundations.LinearModels.ridge_objective_zero_coefficients",
  "MathematicalFoundations.Optimization.scalar_gradient_descent_error_recurrence",
  "MathematicalFoundations.Optimization.scalar_stationary_iteration_succ",
  "MathematicalFoundations.Optimization.scalar_stationary_iteration_tendsto_fixed_point",
]

#check MathematicalFoundations.Linear.linear_map_zero
#check MathematicalFoundations.Linear.linear_map_add
#check MathematicalFoundations.Linear.linear_map_smul
#check MathematicalFoundations.Linear.linear_kernel_zero
#check MathematicalFoundations.Linear.linear_kernel_add
#check MathematicalFoundations.Linear.linear_kernel_smul
#check MathematicalFoundations.Linear.linear_finite_coordinate_reconstruction
#check MathematicalFoundations.Orthogonality.line_projection_residual_decomposition
#check MathematicalFoundations.Orthogonality.line_projection_residual_inner_eq_zero
#check MathematicalFoundations.Orthogonality.symmetric_positive_definite_quadratic_positive
#check MathematicalFoundations.Probability.expectation_add
#check MathematicalFoundations.Probability.expectation_smul
#check MathematicalFoundations.Probability.expectation_sub
#check MathematicalFoundations.Probability.expectation_const
#check MathematicalFoundations.Probability.covariance_expand
#check MathematicalFoundations.BayesInformation.eventProbability_and_comm
#check MathematicalFoundations.BayesInformation.conditional_mul_evidence
#check MathematicalFoundations.BayesInformation.bayes_rule
#check MathematicalFoundations.LinearModels.normal_equation_residual_identity
#check MathematicalFoundations.LinearModels.ridge_objective_zero_coefficients
#check MathematicalFoundations.Optimization.scalar_gradient_descent_error_recurrence
#check MathematicalFoundations.Optimization.scalar_stationary_iteration_succ
#check MathematicalFoundations.Optimization.scalar_stationary_iteration_tendsto_fixed_point

end MathematicalFoundations
