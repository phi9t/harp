import CrouzeixConjecture.FunctionMaximum

namespace CrouzeixTextbook.Part03
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def max_modulus_on_compact_set := @maxFunctionModulusOnSet
set_option linter.defProp false in
def compact_maximum_exists := @exists_maxFunctionModulusOnSet
set_option linter.defProp false in
def pointwise_norm_le_maximum := @norm_function_le_maxFunctionModulusOnSet
set_option linter.defProp false in
def compact_maximum_nonnegative := @maxFunctionModulusOnSet_nonneg
set_option linter.defProp false in
def compact_maximum_monotone := @maxFunctionModulusOnSet_mono
set_option linter.defProp false in
def outer_maxima_converge := @tendsto_maxFunctionModulusOnSet_of_outerApproximation

end
end CrouzeixTextbook.Part03
