import MathematicalFoundations.Orthogonality
import CrouzeixConjecture.CompletionSquareRoot
import CrouzeixConjecture.Positivity

namespace CrouzeixTextbook.Part02
open CrouzeixConjecture

set_option linter.defProp false in
def positive_definite_quadratic_positive := @MathematicalFoundations.Orthogonality.symmetric_positive_definite_quadratic_positive
set_option linter.defProp false in
def gram_matrix_positive := @completionGramMatrix_posSemidef
set_option linter.defProp false in
def invertible_gram_matrix_invertible := @completionGramMatrix_isUnit
set_option linter.defProp false in
def invertible_gram_matrix_positive_definite := @completionGramMatrix_posDef
set_option linter.defProp false in
def positivity_preserved_by_congruence := @posSemidef_congruence
set_option linter.defProp false in
def positive_square_root_data := @completionSquareRootData_of_isUnit

end CrouzeixTextbook.Part02
