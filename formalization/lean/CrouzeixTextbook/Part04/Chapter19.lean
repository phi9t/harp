import CrouzeixConjecture.SimpleSpectrumDensity
import CrouzeixConjecture.SimpleSpectrum

namespace CrouzeixTextbook.Part04
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def simple_spectrum_approximation := @simpleSpectrumApproximation
set_option linter.defProp false in
def simple_spectrum_approximation_distinct := @simpleSpectrumApproximation_hasDistinctEigenvalues
set_option linter.defProp false in
def simple_spectrum_approximation_close := @norm_simpleSpectrumApproximation_sub_lt
set_option linter.defProp false in
def simple_spectrum_approximation_converges := @tendsto_simpleSpectrumApproximation
set_option linter.defProp false in
def distinct_spectrum_dense := @exists_hasDistinctEigenvalues_norm_sub_lt
set_option linter.defProp false in
def diagonal_polynomial_action := @polynomialEval_diagonal

end
end CrouzeixTextbook.Part04
