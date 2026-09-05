import Crouzeix.LoristSchwenninger.Dilation

namespace CrouzeixTextbook.Part04
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def compressed_adjoint_power := @LoristSchwenninger.compressedAdjointPower
set_option linter.defProp false in
def compressed_adjoint_power_contractive := @LoristSchwenninger.DilationData.compressedAdjointPower_norm_le_one
set_option linter.defProp false in
def doubled_compression_norm_two := @LoristSchwenninger.DilationData.doubledCompressedAdjointPower_norm_le_two
set_option linter.defProp false in
def adjacent_power_defect_factorization := @LoristSchwenninger.DilationData.adjacentPowerDefect_factorization
set_option linter.defProp false in
def perturbation_commutes := @LoristSchwenninger.DilationData.perturbation_commutes_with_target
set_option linter.defProp false in
def target_power_bound := @LoristSchwenninger.DilationData.target_power_norm_le_two_add_bound

end
end CrouzeixTextbook.Part04
