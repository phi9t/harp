import Crouzeix.LoristSchwenninger.Recurrence

/-!
An abstract finite-to-limit bridge for the scalar recurrence.

The hypothesis exposes only the weighted inequality available at each finite
horizon.  Its proof uses uniform boundedness to remove the terminal term and
the geometric-series limit to identify the remaining coefficient.
-/

noncomputable section

open Filter Finset Topology

namespace CrouzeixConjecture
namespace Harp

/-- Uniform boundedness and a compatible weighted inequality at every finite
horizon imply the infinite-horizon scalar lower bound. -/
theorem finite_weighted_inequalities_to_limit_lower_bound
    {κ c M : ℝ} {m : ℕ → ℝ}
    (hκ : 1 < κ)
    (hm : ∀ n, |m n| ≤ M)
    (hfinite : ∀ N,
      (κ⁻¹) ^ N * m (N + 1) +
          (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c ≤
        m 1) :
    c / (κ - 1) ≤ m 1 := by
  have hterminal :=
    LoristSchwenninger.bounded_recurrence_terminal_tendsto_zero hκ hm
  have hweights := LoristSchwenninger.inverse_power_weight_sum_tendsto hκ
  have hleft :
      Tendsto
        (fun N : ℕ =>
          (κ⁻¹) ^ N * m (N + 1) +
            (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c)
        atTop (𝓝 ((κ - 1)⁻¹ * c)) := by
    simpa using hterminal.add (hweights.mul_const c)
  have hlimit : (κ - 1)⁻¹ * c ≤ m 1 :=
    le_of_tendsto' hleft hfinite
  simpa [div_eq_mul_inv, mul_comm] using hlimit

end Harp
end CrouzeixConjecture
