import CrouzeixConjecture.MatrixHerglotz

namespace CrouzeixTextbook.Part03
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def open_unit_disk := @openUnitDisk
set_option linter.defProp false in
def unit_circle := @unitCircle
set_option linter.defProp false in
def sampled_kernel_matrix := @sampledKernelMatrix
set_option linter.defProp false in
def positive_matrix_kernel_on := @IsPositiveMatrixKernelOn
set_option linter.defProp false in
def matrix_herglotz_kernel := @matrixHerglotzKernel
set_option linter.defProp false in
def matrix_herglotz_kernel_positive := @matrixHerglotzKernel_isPositiveMatrixKernelOn

end
end CrouzeixTextbook.Part03
