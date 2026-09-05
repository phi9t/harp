import CrouzeixConjecture.CompletionSeries

namespace CrouzeixTextbook.Part04
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def gramian_term := @gramianTerm
set_option linter.defProp false in
def gramian_term_positive := @gramianTerm_posSemidef
set_option linter.defProp false in
def gramian_terms_summable := @summable_gramianTerm
set_option linter.defProp false in
def weighted_gramian := @gramian
set_option linter.defProp false in
def weighted_gramian_positive := @gramian_posSemidef
set_option linter.defProp false in
def gramian_difference_positive := @gramian_two_sub_gramian_four_posSemidef

end
end CrouzeixTextbook.Part04
