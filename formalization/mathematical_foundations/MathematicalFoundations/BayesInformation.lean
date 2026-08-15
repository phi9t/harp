import MathematicalFoundations.Probability
import Mathlib.Tactic.FieldSimp

/-!
Finite-support event weights and Bayes identities.

The definitions below use only finite sums of real weights.  They are not a general
measure-theoretic probability space, and this module contains no entropy, KL-divergence, or
Gaussian claims.  The Bayes theorem explicitly records nonzero prior and evidence weights.
-/

namespace MathematicalFoundations.Probability

open scoped BigOperators

/-- The total weight of an event restricted to an explicit finite support. -/
def eventWeight {α : Type*} (support : Finset α) (weight : α → ℝ) (event : α → Prop)
    [DecidablePred event] : ℝ :=
  ∑ x ∈ support.filter event, weight x

/-- The finite-support conditional weight of `event` given `evidence`. -/
noncomputable def conditionalProbability {α : Type*} (support : Finset α) (weight : α → ℝ)
    (event evidence : α → Prop) [DecidablePred event] [DecidablePred evidence] : ℝ :=
  eventWeight support weight (fun x => event x ∧ evidence x) / eventWeight support weight evidence

/-- Conjunction does not change an event weight when its factors are exchanged. -/
theorem eventWeight_and_comm {α : Type*} (support : Finset α) (weight : α → ℝ)
    (a b : α → Prop) [DecidablePred a] [DecidablePred b] :
    eventWeight support weight (fun x => a x ∧ b x) =
      eventWeight support weight (fun x => b x ∧ a x) := by
  unfold eventWeight
  congr 1
  ext x
  simp only [Finset.mem_filter]
  constructor
  · rintro ⟨hx, ha, hb⟩
    exact ⟨hx, hb, ha⟩
  · rintro ⟨hx, hb, ha⟩
    exact ⟨hx, ha, hb⟩

/-- With nonzero evidence weight, conditional weight times evidence weight reconstructs the
joint event weight. -/
theorem conditional_mul_evidence {α : Type*} (support : Finset α) (weight : α → ℝ)
    (event evidence : α → Prop) [DecidablePred event] [DecidablePred evidence]
    (hevidence : eventWeight support weight evidence ≠ 0) :
    conditionalProbability support weight event evidence * eventWeight support weight evidence =
      eventWeight support weight (fun x => event x ∧ evidence x) := by
  unfold conditionalProbability
  exact div_mul_cancel₀ _ hevidence

/-- Bayes' rule for finite-support real weights, assuming nonzero prior and evidence weights. -/
theorem bayes_rule {α : Type*} (support : Finset α) (weight : α → ℝ)
    (a b : α → Prop) [DecidablePred a] [DecidablePred b]
    (hprior : eventWeight support weight a ≠ 0) (hevidence : eventWeight support weight b ≠ 0) :
    conditionalProbability support weight a b =
      conditionalProbability support weight b a * eventWeight support weight a /
        eventWeight support weight b := by
  unfold conditionalProbability
  rw [eventWeight_and_comm support weight a b]
  field_simp [hprior, hevidence]

end MathematicalFoundations.Probability
