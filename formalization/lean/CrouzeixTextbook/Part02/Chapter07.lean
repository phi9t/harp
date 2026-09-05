import MathematicalFoundations.Orthogonality
import CrouzeixConjecture.Euclidean

namespace CrouzeixTextbook.Part02
open CrouzeixConjecture

set_option linter.defProp false in
def projection_residual_decomposition := @MathematicalFoundations.Orthogonality.line_projection_residual_decomposition
set_option linter.defProp false in
def projection_residual_orthogonal := @MathematicalFoundations.Orthogonality.line_projection_residual_inner_eq_zero
set_option linter.defProp false in
def adjoint_coordinate_identity := @inner_euclideanOperator_eq_star_dotProduct
set_option linter.defProp false in
def adjoint_matrix_is_conjugate_transpose := @euclideanOperator_conjTranspose
set_option linter.defProp false in
def matrix_norm_is_operator_norm := @matrix_norm_eq_euclidean_operator_norm

theorem norm_is_nonnegative {E : Type*} [SeminormedAddGroup E] (x : E) : 0 ≤ ‖x‖ := norm_nonneg x

end CrouzeixTextbook.Part02
