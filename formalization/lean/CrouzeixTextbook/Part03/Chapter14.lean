import CrouzeixConjecture.MatrixPowerSeries

namespace CrouzeixTextbook.Part03
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def matrix_power_series := @matrixPowerSeries
set_option linter.defProp false in
def matrix_power_series_coefficient := @matrixPowerSeries_apply
set_option linter.defProp false in
def matrix_power_series_radius := @one_le_matrixPowerSeries_radius
set_option linter.defProp false in
def matrix_power_series_sum := @matrixPowerSeriesSum
set_option linter.defProp false in
def matrix_power_series_analytic := @matrixPowerSeriesSum_analyticOnNhd_unitDisk
set_option linter.defProp false in
def matrix_power_series_converges := @matrixPowerSeries_hasSum

end
end CrouzeixTextbook.Part03
