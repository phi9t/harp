import CrouzeixConjecture.RationalFunctionalCalculus

namespace CrouzeixTextbook.Part03
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def rational_pole_set := @rationalPoleSet
set_option linter.defProp false in
def rational_poles_finite := @rationalPoleSet_finite
set_option linter.defProp false in
def pole_complement_open := @isOpen_compl_rationalPoleSet
set_option linter.defProp false in
def rational_pole_free_on := @RationalPoleFreeOn
set_option linter.defProp false in
def rational_scalar_eval := @rationalScalarEval
set_option linter.defProp false in
def rational_matrix_eval := @rationalMatrixEval

end
end CrouzeixTextbook.Part03
