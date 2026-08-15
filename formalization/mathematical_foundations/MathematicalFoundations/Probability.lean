import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring

/-!
Finite-support weighted expectation and covariance identities.

All sums in this module are explicitly over a `Finset`, and weights are arbitrary real
coefficients unless a theorem states a normalization hypothesis.  This module deliberately
does not define measure-theoretic probability, prove convergence, or make entropy, KL, or
Gaussian-distribution claims.
-/

namespace MathematicalFoundations.Probability

open scoped BigOperators

/-- The weighted expectation of a real function over an explicit finite support. -/
def expectation {α : Type*} (support : Finset α) (weight value : α → ℝ) : ℝ :=
  ∑ x ∈ support, weight x * value x

/-- Weighted expectation distributes over pointwise addition on a finite support. -/
theorem expectation_add {α : Type*} (support : Finset α) (weight f g : α → ℝ) :
    expectation support weight (fun x => f x + g x) =
      expectation support weight f + expectation support weight g := by
  simp only [expectation, mul_add, Finset.sum_add_distrib]

/-- A real scalar can be pulled out of a finite weighted expectation. -/
theorem expectation_smul {α : Type*} (support : Finset α) (weight f : α → ℝ) (c : ℝ) :
    expectation support weight (fun x => c * f x) = c * expectation support weight f := by
  unfold expectation
  calc
    ∑ x ∈ support, weight x * (c * f x) =
        ∑ x ∈ support, c * (weight x * f x) := by
          apply Finset.sum_congr rfl
          intro x hx
          ring
    _ = c * ∑ x ∈ support, weight x * f x := by
      rw [Finset.mul_sum]

/-- Weighted expectation distributes over pointwise subtraction on a finite support. -/
theorem expectation_sub {α : Type*} (support : Finset α) (weight f g : α → ℝ) :
    expectation support weight (fun x => f x - g x) =
      expectation support weight f - expectation support weight g := by
  unfold expectation
  calc
    ∑ x ∈ support, weight x * (f x - g x) =
        ∑ x ∈ support, (weight x * f x - weight x * g x) := by
          apply Finset.sum_congr rfl
          intro x hx
          ring
    _ = (∑ x ∈ support, weight x * f x) - ∑ x ∈ support, weight x * g x := by
      rw [Finset.sum_sub_distrib]

/-- A normalized finite weight assigns a constant its constant expectation. -/
theorem expectation_const {α : Type*} (support : Finset α) (weight : α → ℝ) (c : ℝ)
    (hnormalized : ∑ x ∈ support, weight x = 1) :
    expectation support weight (fun _ => c) = c := by
  unfold expectation
  calc
    ∑ x ∈ support, weight x * c = ∑ x ∈ support, c * weight x := by
      apply Finset.sum_congr rfl
      intro x hx
      ring
    _ = c * ∑ x ∈ support, weight x := by
      rw [Finset.mul_sum]
    _ = c := by rw [hnormalized, mul_one]

/-- The finite weighted covariance defined from the corresponding weighted expectations. -/
def covariance {α : Type*} (support : Finset α) (weight f g : α → ℝ) : ℝ :=
  expectation support weight
    (fun x => (f x - expectation support weight f) * (g x - expectation support weight g))

/-- A normalized finite weighted covariance is expectation of the product minus product of
expectations. -/
theorem covariance_expand {α : Type*} (support : Finset α) (weight f g : α → ℝ)
    (hnormalized : ∑ x ∈ support, weight x = 1) :
    covariance support weight f g =
      expectation support weight (fun x => f x * g x) -
        expectation support weight f * expectation support weight g := by
  let meanF := expectation support weight f
  let meanG := expectation support weight g
  calc
    covariance support weight f g =
        expectation support weight (fun x =>
          f x * g x - meanG * f x - meanF * g x + meanF * meanG) := by
            unfold covariance
            apply congrArg (expectation support weight)
            funext x
            dsimp [meanF, meanG]
            ring
    _ = (expectation support weight (fun x => f x * g x) -
          expectation support weight (fun x => meanG * f x)) -
          expectation support weight (fun x => meanF * g x) +
          expectation support weight (fun _ => meanF * meanG) := by
            rw [expectation_add, expectation_sub, expectation_sub]
    _ = (expectation support weight (fun x => f x * g x) -
          meanG * expectation support weight f) -
          meanF * expectation support weight g + meanF * meanG := by
            rw [expectation_smul, expectation_smul,
              expectation_const support weight (meanF * meanG) hnormalized]
    _ = expectation support weight (fun x => f x * g x) -
          expectation support weight f * expectation support weight g := by
            dsimp [meanF, meanG]
            ring

end MathematicalFoundations.Probability
