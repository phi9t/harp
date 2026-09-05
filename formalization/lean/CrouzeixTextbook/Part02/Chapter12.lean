import CrouzeixConjecture.RadialOrientation
import CrouzeixConjecture.ParallelRadialDifferentiability

namespace CrouzeixTextbook.Part02
open CrouzeixConjecture

set_option linter.defProp false in
def real_inner_complex_coordinates := @complexRealInner_eq_re_mul_conj
set_option linter.defProp false in
def area_form_quarter_turn := @complexAreaForm_I_mul_right
set_option linter.defProp false in
def oriented_radial_tangent_positive := @complexAreaForm_radialTangent_pos
set_option linter.defProp false in
def oriented_tangent_formula := @radialTangent_eq_I_mul_normal_mul_norm
set_option linter.defProp false in
def boundary_parameter_is_smooth := @contDiff_one_parallelRadialParameterPoint
set_option linter.defProp false in
def boundary_slice_derivative := @parallelRadialParameterPoint_slice_hasDerivAt

end CrouzeixTextbook.Part02
