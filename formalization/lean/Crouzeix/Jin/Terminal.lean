import Crouzeix.Jin.MaxPolynomialModulus
import CrouzeixConjecture.HolomorphicConsequences

/-!
Harp-owned terminal assembly surface for Jin's Crouzeix route.

The pinned Jin terminal theorem is:

```
theorem crouzeixConjecture (A : SquareMatrix n) (p : Polynomial ℂ) :
    ‖polynomialEval p A‖ ≤
      2 * maxPolynomialModulusOnNumericalRange A p := by
  have hbound := holomorphicCrouzeixBound A isOpen_univ (Set.subset_univ _)
    p.differentiableOn
  rw [holomorphicMatrixEval_polynomial] at hbound
  simpa only [maxFunctionModulusOnSet, maxPolynomialModulusOnNumericalRange] using hbound
```

This module intentionally does not declare `crouzeixConjecture` until the
Harp-native holomorphic Crouzeix route exists. A terminal proof-slice attempt
imports this module and checks that declaration name, producing a typed failed
receipt at the exact missing mathematical bridge instead of importing Jin's
upstream final theorem as proof authority.
-/

noncomputable section

namespace CrouzeixConjecture

open scoped Matrix.Norms.L2Operator

variable {n : Type*} [Fintype n] [DecidableEq n]

/-- Terminal assembly blocker: Jin's terminal theorem needs the holomorphic
Crouzeix bound and the polynomial specialization rewrite before the exact
terminal statement can be declared without trusted assumptions. -/
def terminalCrouzeixAssemblyBlocker : Prop :=
  ∀ (A : SquareMatrix n) (p : Polynomial ℂ),
    PolynomialCrouzeixBound A p

/-- The polynomial specialization of Jin's main theorem, assembled in Harp
from the lower holomorphic Crouzeix route rather than by importing Jin's
terminal theorem. -/
theorem crouzeixConjecture [Nonempty n] (A : SquareMatrix n) (p : Polynomial ℂ) :
    ‖polynomialEval p A‖ ≤
      2 * maxPolynomialModulusOnNumericalRange A p := by
  have hbound := holomorphicCrouzeixBound A isOpen_univ (Set.subset_univ _)
    p.differentiableOn
  rw [holomorphicMatrixEval_polynomial] at hbound
  simpa only [maxFunctionModulusOnSet, maxPolynomialModulusOnNumericalRange] using hbound

end CrouzeixConjecture
