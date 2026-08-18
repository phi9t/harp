import MathematicalFoundations.Probability
import Mathlib.Tactic.FieldSimp

/-!
Finite-PMF event probabilities and Bayes identities.

Every definition below consumes a `FiniteProbabilityMass`, whose masses are nonnegative and
sum to one over an explicit `Finset` support.  This is not a measure-theoretic probability
space, and this module contains no entropy, KL-divergence, or Gaussian claims.
-/

namespace MathematicalFoundations.BayesInformation

open scoped BigOperators

/-- The probability of an event under an explicit finite probability mass function. -/
def eventProbability {α : Type*} (distribution : Probability.FiniteProbabilityMass α)
    (event : α → Prop) [DecidablePred event] : ℝ :=
  ∑ x ∈ distribution.support.filter event, distribution.mass x

/-- The finite-PMF conditional probability of `event` given `evidence`. -/
noncomputable def conditionalProbability {α : Type*}
    (distribution : Probability.FiniteProbabilityMass α) (event evidence : α → Prop)
    [DecidablePred event] [DecidablePred evidence] : ℝ :=
  eventProbability distribution (fun x => event x ∧ evidence x) /
    eventProbability distribution evidence

/-- Exchanging conjunction factors does not change a finite-PMF event probability. -/
theorem eventProbability_and_comm {α : Type*} (distribution : Probability.FiniteProbabilityMass α)
    (a b : α → Prop) [DecidablePred a] [DecidablePred b] :
    eventProbability distribution (fun x => a x ∧ b x) =
      eventProbability distribution (fun x => b x ∧ a x) := by
  unfold eventProbability
  congr 1
  ext x
  simp only [Finset.mem_filter]
  constructor
  · rintro ⟨hx, ha, hb⟩
    exact ⟨hx, hb, ha⟩
  · rintro ⟨hx, hb, ha⟩
    exact ⟨hx, ha, hb⟩

/-- With nonzero evidence probability, conditional probability times evidence probability
reconstructs the joint-event probability. -/
theorem conditional_mul_evidence {α : Type*} (distribution : Probability.FiniteProbabilityMass α)
    (event evidence : α → Prop) [DecidablePred event] [DecidablePred evidence]
    (hevidence : eventProbability distribution evidence ≠ 0) :
    conditionalProbability distribution event evidence * eventProbability distribution evidence =
      eventProbability distribution (fun x => event x ∧ evidence x) := by
  unfold conditionalProbability
  exact div_mul_cancel₀ _ hevidence

/-- Bayes' rule for finite probability mass functions with nonzero prior and evidence
probabilities. -/
theorem bayes_rule {α : Type*} (distribution : Probability.FiniteProbabilityMass α)
    (a b : α → Prop) [DecidablePred a] [DecidablePred b]
    (hprior : eventProbability distribution a ≠ 0)
    (hevidence : eventProbability distribution b ≠ 0) :
    conditionalProbability distribution a b =
      conditionalProbability distribution b a * eventProbability distribution a /
        eventProbability distribution b := by
  unfold conditionalProbability
  rw [eventProbability_and_comm distribution a b]
  field_simp [hprior, hevidence]

end MathematicalFoundations.BayesInformation
