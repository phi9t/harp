import Crouzeix.LoristSchwenninger.DisplacementUpper
import Crouzeix.LoristSchwenninger.OperatorRecurrence
import Crouzeix.LoristSchwenninger.Scalar

/-!
Assembly of the Lorist--Schwenninger perturbation lemma.

This module combines norm attainment, the operator recurrence, source
Equations (3) and (4), and the terminal scalar contradiction.
-/

noncomputable section

open scoped InnerProduct

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {E K : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace ℂ E]
  [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E]
variable [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]

namespace DilationData

/-- The source-faithful Lorist--Schwenninger perturbation lemma. -/
theorem norm_target_le_two (data : DilationData (E := E) (K := K)) :
    ‖data.T‖ ≤ 2 := by
  by_cases hsmall : ‖data.T‖ ≤ 1
  · linarith
  · have hnorm : 1 < ‖data.T‖ := lt_of_not_ge hsmall
    obtain ⟨x, hx, _hTx, hsingular⟩ :=
      exists_unit_norm_attaining_and_adjoint_apply data.T
    have hlower := data.equation_three_lower_bound hnorm hx hsingular
    have hlower' :
        -data.displacementSq ‖data.T‖ x /
            (‖data.T‖ * (‖data.T‖ - 1) ^ 2) ≤
          data.firstPerturbationMoment x := by
      have hreal :
          (inner ℂ x (data.perturbation 1 (data.T x))).re =
            (inner ℂ (data.perturbation 1 (data.T x)) x).re :=
        @inner_re_symm ℂ E _ _ _ x (data.perturbation 1 (data.T x))
      exact hlower.trans_eq hreal
    have hupper := data.displacementSq_le hx rfl hsingular
    exact scalar_endpoint_le_two
      (data.displacementSq_nonneg ‖data.T‖ x)
      (lt_trans zero_lt_one hnorm)
      (sq_pos_of_pos (sub_pos.mpr hnorm))
      hlower' hupper

end DilationData
end LoristSchwenninger
end CrouzeixConjecture
