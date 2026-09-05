import AutodiffGeometry.FiniteCoordinates
import CrouzeixTextbook.Part01.Chapter05

namespace CrouzeixTextbook.Part02

set_option linter.defProp false in
def bilinear_pairing_duality := @AutodiffGeometry.dot_jvp_eq_dot_vjp
set_option linter.defProp false in
def transpose_coordinate_action := @AutodiffGeometry.vjp_eq_transpose_matVec
set_option linter.defProp false in
def determinant_top_degree_multiplicative := @CrouzeixTextbook.Part01.determinant_multiplicative
set_option linter.defProp false in
def alternating_diagonal_volume := @CrouzeixTextbook.Part01.determinant_diagonal
set_option linter.defProp false in
def contraction_trace_cyclic := @CrouzeixTextbook.Part01.trace_cyclic

theorem wedge_sign_kernel {a b : ℤ} : -(a - b) = b - a := by ring

end CrouzeixTextbook.Part02
