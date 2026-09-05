import CrouzeixConjecture.HolomorphicOuterLimit

namespace CrouzeixTextbook.Part03
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def holomorphic_matrix_eval := @holomorphicMatrixEval
set_option linter.defProp false in
def contour_eval_agrees := @PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_holomorphicMatrixEval
set_option linter.defProp false in
def polynomial_compatibility := @holomorphicMatrixEval_polynomial
set_option linter.defProp false in
def locality_on_neighborhood := @holomorphicMatrixEval_congr_on_neighborhood
set_option linter.defProp false in
def functional_calculus_additive := @holomorphicMatrixEval_add
set_option linter.defProp false in
def functional_calculus_multiplicative := @holomorphicMatrixEval_mul

end
end CrouzeixTextbook.Part03
