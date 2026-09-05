import CrouzeixConjecture.NumericalRangeConvexity
import CrouzeixConjecture.NumericalRange
import CrouzeixConjecture.Spectrum

namespace CrouzeixTextbook.Part04
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def numerical_range_nonempty := @numericalRange_nonempty
set_option linter.defProp false in
def numerical_range_as_sphere_image := @numericalRange_eq_image_sphere
set_option linter.defProp false in
def numerical_range_compact := @isCompact_numericalRange
set_option linter.defProp false in
def numerical_range_convex := @numericalRange_convex
set_option linter.defProp false in
def numerical_range_perturbation_bound := @numericalRange_perturbation
set_option linter.defProp false in
def spectrum_lies_in_numerical_range := @matrixSpectrum_subset_numericalRange

end
end CrouzeixTextbook.Part04
