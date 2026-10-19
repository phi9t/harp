import CrouzeixConjecture.DoubleLayerPositiveMap

namespace CrouzeixTextbook.Part04
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def boundary_positive_map := @boundaryPhi
set_option linter.defProp false in
def boundary_positive_linear_map := @boundaryPhiLinear
set_option linter.defProp false in
def boundary_positive_map_norm := @boundaryPhi_norm_le
set_option linter.defProp false in
def boundary_positive_map_unital := @boundaryPhi_one
set_option linter.defProp false in
def boundary_positive_map_star := @boundaryPhi_star
set_option linter.defProp false in
def boundary_positive_map_preserves_psd := @boundaryPhi_posSemidef

end
end CrouzeixTextbook.Part04
