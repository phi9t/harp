import CrouzeixConjecture.CompletionSquareRoot
import CrouzeixConjecture.Positivity

namespace CrouzeixTextbook.Part02
open CrouzeixConjecture

set_option linter.defProp false in
def induced_matrix_norm_identity := @matrix_norm_eq_euclidean_operator_norm
set_option linter.defProp false in
def polar_factor_is_unitary := @completionPolarUnitary_mem_unitaryGroup
set_option linter.defProp false in
def polar_similarity_norm_transfer := @completionDiagonalizableMatrix_norm_eq_completionSimilarity_norm
set_option linter.defProp false in
def polar_operator_norm_transfer := @completionDiagonalizableMatrix_euclideanOperator_norm_eq
set_option linter.defProp false in
def quadratic_bound_implies_norm_two := @matrix_norm_le_two_of_four_sub_conjTranspose_mul_self_posSemidef

theorem norm_triangle_kernel {E : Type*} [SeminormedAddGroup E] (x y : E) : ‖x + y‖ ≤ ‖x‖ + ‖y‖ := norm_add_le x y

end CrouzeixTextbook.Part02
