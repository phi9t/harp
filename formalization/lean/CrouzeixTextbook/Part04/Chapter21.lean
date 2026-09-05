import CrouzeixConjecture.HilbertSpectralSetCore

namespace CrouzeixTextbook.Part04
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def operator_numerical_range_convex := @operatorNumericalRange_convex
set_option linter.defProp false in
def closed_operator_numerical_range := @closedOperatorNumericalRange
set_option linter.defProp false in
def closed_operator_numerical_range_nonempty := @closedOperatorNumericalRange_nonempty
set_option linter.defProp false in
def closed_operator_numerical_range_compact := @closedOperatorNumericalRange_isCompact
set_option linter.defProp false in
def hilbert_rational_spectral_set_statement := @HilbertRationalSpectralSetStatement
set_option linter.defProp false in
def closed_numerical_range_two_spectral_set := @ClosedOperatorNumericalRangeIsTwoSpectralSet

end
end CrouzeixTextbook.Part04
