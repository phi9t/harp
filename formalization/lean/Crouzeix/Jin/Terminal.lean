module

public import Crouzeix.Jin.MaxPolynomialModulus
public import CrouzeixConjecture.HolomorphicConsequences

@[expose] public section

/-!
Harp-owned terminal assembly surface for Jin's Crouzeix route.

The pinned Jin terminal theorem is:

```
theorem crouzeixConjecture [Nonempty n] (A : SquareMatrix n) (p : Polynomial ℂ) :
    ‖polynomialEval p A‖ ≤
      2 * maxPolynomialModulusOnNumericalRange A p := by
  have hbound := holomorphicCrouzeixBound A isOpen_univ (Set.subset_univ _)
    p.differentiableOn
  rw [holomorphicMatrixEval_polynomial] at hbound
  simpa only [maxFunctionModulusOnSet, maxPolynomialModulusOnNumericalRange] using hbound
```

This module declares `crouzeixConjecture` from the completed Harp-native
holomorphic route. The assembly uses Harp-owned intermediate theorems rather
than importing Jin's upstream final theorem as proof authority.
-/

noncomputable section

namespace CrouzeixConjecture

open scoped Matrix.Norms.L2Operator

variable {n : Type*} [Fintype n] [DecidableEq n]

/-- The exact terminal assembly obligation discharged by the Harp-owned
holomorphic bound and polynomial specialization. -/
def terminalCrouzeixAssemblyBlocker : Prop :=
  ∀ (A : SquareMatrix n) (p : Polynomial ℂ),
    PolynomialCrouzeixBound A p

/-- The polynomial specialization of Jin's main theorem, assembled in Harp
from the lower holomorphic Crouzeix route rather than by importing Jin's
terminal theorem. -/
theorem crouzeixConjecture [Nonempty n] (A : SquareMatrix n) (p : Polynomial ℂ) :
    ‖polynomialEval p A‖ ≤
      2 * maxPolynomialModulusOnNumericalRange A p :=
  polynomialCrouzeixBound_of_holomorphicCrouzeixBound A p

end CrouzeixConjecture
