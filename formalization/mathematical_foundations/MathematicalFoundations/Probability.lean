import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring

/-!
Finite-probability-mass-function expectation and covariance identities.

Every public probability construction in this module consumes a `FiniteProbabilityMass`: an
explicit `Finset` support, nonnegative real masses on that support, and total mass one.  This
module deliberately does not define measure-theoretic probability, prove convergence, or make
entropy, KL, or Gaussian-distribution claims.
-/

namespace MathematicalFoundations.Probability

open scoped BigOperators

/-- A real-valued probability mass function with an explicit finite support. -/
structure FiniteProbabilityMass (α : Type*) where
  /-- The finite carrier over which all probabilities and expectations are summed. -/
  support : Finset α
  /-- The mass assigned to each element of the support. -/
  mass : α → ℝ
  /-- Masses on the explicit support are nonnegative. -/
  nonnegative : ∀ x ∈ support, 0 ≤ mass x
  /-- The masses over the explicit support total one. -/
  total_mass : ∑ x ∈ support, mass x = 1

/-- The expectation of a real function under a finite probability mass function. -/
def expectation {α : Type*} (distribution : FiniteProbabilityMass α) (value : α → ℝ) : ℝ :=
  ∑ x ∈ distribution.support, distribution.mass x * value x

/-- Finite-PMF expectation distributes over pointwise addition. -/
theorem expectation_add {α : Type*} (distribution : FiniteProbabilityMass α) (f g : α → ℝ) :
    expectation distribution (fun x => f x + g x) =
      expectation distribution f + expectation distribution g := by
  simp only [expectation, mul_add, Finset.sum_add_distrib]

/-- A real scalar can be pulled out of a finite-PMF expectation. -/
theorem expectation_smul {α : Type*} (distribution : FiniteProbabilityMass α)
    (f : α → ℝ) (c : ℝ) :
    expectation distribution (fun x => c * f x) = c * expectation distribution f := by
  unfold expectation
  calc
    ∑ x ∈ distribution.support, distribution.mass x * (c * f x) =
        ∑ x ∈ distribution.support, c * (distribution.mass x * f x) := by
          apply Finset.sum_congr rfl
          intro x hx
          ring
    _ = c * ∑ x ∈ distribution.support, distribution.mass x * f x := by
      rw [Finset.mul_sum]

/-- Finite-PMF expectation distributes over pointwise subtraction. -/
theorem expectation_sub {α : Type*} (distribution : FiniteProbabilityMass α) (f g : α → ℝ) :
    expectation distribution (fun x => f x - g x) =
      expectation distribution f - expectation distribution g := by
  unfold expectation
  calc
    ∑ x ∈ distribution.support, distribution.mass x * (f x - g x) =
        ∑ x ∈ distribution.support,
          (distribution.mass x * f x - distribution.mass x * g x) := by
            apply Finset.sum_congr rfl
            intro x hx
            ring
    _ = (∑ x ∈ distribution.support, distribution.mass x * f x) -
          ∑ x ∈ distribution.support, distribution.mass x * g x := by
            rw [Finset.sum_sub_distrib]

/-- A constant has its own value as finite-PMF expectation. -/
theorem expectation_const {α : Type*} (distribution : FiniteProbabilityMass α) (c : ℝ) :
    expectation distribution (fun _ => c) = c := by
  unfold expectation
  calc
    ∑ x ∈ distribution.support, distribution.mass x * c =
        ∑ x ∈ distribution.support, c * distribution.mass x := by
          apply Finset.sum_congr rfl
          intro x hx
          ring
    _ = c * ∑ x ∈ distribution.support, distribution.mass x := by
      rw [Finset.mul_sum]
    _ = c := by rw [distribution.total_mass, mul_one]

/-- The covariance under a finite probability mass function. -/
def covariance {α : Type*} (distribution : FiniteProbabilityMass α) (f g : α → ℝ) : ℝ :=
  expectation distribution
    (fun x => (f x - expectation distribution f) * (g x - expectation distribution g))

/-- Finite-PMF covariance is expectation of the product minus product of expectations. -/
theorem covariance_expand {α : Type*} (distribution : FiniteProbabilityMass α) (f g : α → ℝ) :
    covariance distribution f g =
      expectation distribution (fun x => f x * g x) -
        expectation distribution f * expectation distribution g := by
  let meanF := expectation distribution f
  let meanG := expectation distribution g
  calc
    covariance distribution f g =
        expectation distribution (fun x =>
          f x * g x - meanG * f x - meanF * g x + meanF * meanG) := by
            unfold covariance
            apply congrArg (expectation distribution)
            funext x
            dsimp [meanF, meanG]
            ring
    _ = (expectation distribution (fun x => f x * g x) -
          expectation distribution (fun x => meanG * f x)) -
          expectation distribution (fun x => meanF * g x) +
          expectation distribution (fun _ => meanF * meanG) := by
            rw [expectation_add, expectation_sub, expectation_sub]
    _ = (expectation distribution (fun x => f x * g x) -
          meanG * expectation distribution f) -
          meanF * expectation distribution g + meanF * meanG := by
            rw [expectation_smul, expectation_smul, expectation_const]
    _ = expectation distribution (fun x => f x * g x) -
          expectation distribution f * expectation distribution g := by
            dsimp [meanF, meanG]
            ring

end MathematicalFoundations.Probability
